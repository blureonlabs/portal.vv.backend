use std::sync::Arc;

use actix_web::{web, App, HttpServer, HttpResponse};
use actix_governor::{Governor, GovernorConfigBuilder};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use database::domain::DatabasePool;

mod config;
mod common;
mod database;
mod auth;
mod driver;
mod vehicle;
mod trip;
mod finance;
mod advance;
mod salary;
mod hr;
mod invoice;
mod notification;
mod report;
mod settings;
mod audit;
mod platform;
mod owner;
mod portal;
mod comms;
mod document;

use config::AppConfig;
use database::infrastructure::PgDatabase;

use audit::application::service::AuditService;
use notification::{
    infrastructure::ResendClient,
    application::service::NotificationService,
};
use common::deps::SharedDeps;
use common::ports::DeductionPort;

#[derive(Parser)]
#[command(name = "fms", about = "Fleet Management System — UAE Operations")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Bootstrap the first Super Admin. Run once, then remove SEED_KEY from env.
    SeedAdmin {
        #[arg(long)]
        email: String,
    },
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "fms=info,actix_web=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env()?;
    let db = PgDatabase::connect(&config.database_url).await?;
    db.run_migrations().await?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::SeedAdmin { email }) => {
            auth::application::service::seed_admin(db.pg_pool(), &email, &config).await?;
        }
        None => {
            start_server(config, db).await?;
        }
    }

    Ok(())
}

async fn start_server(config: AppConfig, db: PgDatabase) -> anyhow::Result<()> {
    let addr = config.addr();
    info!("Starting FMS on {}", addr);

    let config = Arc::new(config);

    // ── Core shared deps ─────────────────────────────────────────────────────
    let resend = Arc::new(ResendClient::new(&config));
    let notification_svc = Arc::new(NotificationService::new(Arc::clone(&resend)));
    let audit_svc = Arc::new(AuditService::new(db.pg_pool().clone()));

    let deps = SharedDeps {
        pool: db.pg_pool().clone(),
        config: Arc::clone(&config),
        audit: Arc::clone(&audit_svc),
        notification: Arc::clone(&notification_svc),
    };

    // ── Build per-feature deps (once, outside the worker closure) ────────────
    let auth_deps = auth::configure::build(&deps);
    let driver_deps = driver::configure::build(&deps);
    let vehicle_deps = vehicle::configure::build(&deps);
    let trip_deps = trip::configure::build(&deps);
    let finance_deps = finance::configure::build(&deps);
    let advance_deps = advance::configure::build(&deps);
    let hr_deps = hr::configure::build(&deps);
    let invoice_deps = invoice::configure::build(&deps).await?;
    let settings_deps = settings::configure::build(&deps);
    let report_deps = report::configure::build(&deps);
    let audit_deps = audit::configure::build(Arc::clone(&audit_svc));
    let owner_deps = owner::configure::build(&deps);
    let comms_deps = comms::configure::build(&deps);
    let document_deps = document::configure::build(&deps);
    let notification_deps = notification::configure::build(Arc::clone(&notification_svc));
    let platform_deps = platform::configure::build(&deps);

    // Cross-feature wiring: salary needs advance (DeductionPort) + settings repo
    let deduction_port: Arc<dyn DeductionPort> = advance_deps.repo.clone();
    let salary_deps = salary::configure::build(&deps, deduction_port, Arc::clone(&settings_deps.repo));

    // ── Data handles for items registered at app-level ───────────────────────
    let config_data = web::Data::new((*config).clone());
    let db_data = web::Data::new(db);

    let governor_conf = GovernorConfigBuilder::default()
        .seconds_per_request(2)
        .burst_size(30)
        .finish()
        .unwrap();

    let server = HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allowed_origin("https://portal.voiturevoyages.com")
            .allowed_origin("http://localhost:5173")  // dev
            .allowed_origin("http://localhost:3000")  // dev
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Governor::new(&governor_conf))
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(config_data.clone())
            .app_data(db_data.clone())
            .route("/", web::get().to(|| async { HttpResponse::Ok().body("FMS OK") }))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .configure(|cfg| auth::configure::register(&auth_deps, cfg))
                    .configure(|cfg| driver::configure::register(&driver_deps, cfg))
                    .configure(|cfg| vehicle::configure::register(&vehicle_deps, cfg))
                    .configure(|cfg| trip::configure::register(&trip_deps, cfg))
                    .configure(|cfg| finance::configure::register(&finance_deps, cfg))
                    .configure(|cfg| advance::configure::register(&advance_deps, cfg))
                    .configure(|cfg| salary::configure::register(&salary_deps, cfg))
                    .configure(|cfg| hr::configure::register(&hr_deps, cfg))
                    .configure(|cfg| invoice::configure::register(&invoice_deps, cfg))
                    .configure(|cfg| report::configure::register(&report_deps, cfg))
                    .configure(|cfg| settings::configure::register(&settings_deps, cfg))
                    .configure(|cfg| audit::configure::register(&audit_deps, cfg))
                    .configure(|cfg| platform::configure::register(&platform_deps, cfg))
                    .configure(|cfg| owner::configure::register(&owner_deps, cfg))
                    .configure(portal::configure::register)
                    .configure(|cfg| comms::configure::register(&comms_deps, cfg))
                    .configure(|cfg| document::configure::register(&document_deps, cfg))
                    .configure(|cfg| notification::configure::register(&notification_deps, cfg))
            )
    })
    .bind(&addr)?;

    info!("Server bound successfully to {}", addr);

    // Keep-alive: prevent Render free-tier spin-down (ticks every 14 min)
    tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(840));
        loop {
            interval.tick().await;
            tracing::debug!("keep-alive tick");
        }
    });

    server.run().await?;

    Ok(())
}

async fn health_check(db: web::Data<PgDatabase>) -> HttpResponse {
    use crate::database::domain::DatabasePool;
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(db.pg_pool())
        .await
        .is_ok();

    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "db": if db_ok { "ok" } else { "error" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
