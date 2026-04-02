use std::sync::Arc;

use actix_web::{web, App, HttpServer, HttpResponse};
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
mod uber;

use config::AppConfig;
use database::infrastructure::PgDatabase;

use auth::{
    domain::repository::AuthRepository,
    infrastructure::{PgAuthRepository, SupabaseAdminClient},
    application::service::AuthService,
};
use notification::{
    infrastructure::ResendClient,
    application::service::NotificationService,
};
use audit::application::service::AuditService;
use driver::{
    domain::repository::DriverRepository,
    infrastructure::PgDriverRepository,
    application::service::DriverService,
};
use vehicle::{
    domain::repository::VehicleRepository,
    infrastructure::PgVehicleRepository,
    application::service::VehicleService,
};
use trip::{
    infrastructure::PgTripRepository,
    application::service::TripService,
};
use finance::{
    infrastructure::PgFinanceRepository,
    application::service::FinanceService,
};

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
            .unwrap_or_else(|_| "fms=debug,actix_web=info".into()))
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

    // ── Build shared services ─────────────────────────────────────────────────
    let resend = Arc::new(ResendClient::new(&config));
    let notification_svc = Arc::new(NotificationService::new(resend));
    let audit_svc = Arc::new(AuditService::new(db.pg_pool().clone()));

    let auth_repo: Arc<dyn AuthRepository> = Arc::new(PgAuthRepository::new(db.pg_pool().clone()));
    let supabase = Arc::new(SupabaseAdminClient::new(&config));
    let auth_svc = Arc::new(AuthService::new(
        Arc::clone(&auth_repo),
        supabase,
        Arc::clone(&config),
        Arc::clone(&notification_svc),
        Arc::clone(&audit_svc),
    ));

    let driver_repo: Arc<dyn DriverRepository> = Arc::new(PgDriverRepository::new(db.pg_pool().clone()));
    let driver_svc = Arc::new(DriverService::new(Arc::clone(&driver_repo), Arc::clone(&audit_svc)));

    let vehicle_repo: Arc<dyn VehicleRepository> = Arc::new(PgVehicleRepository::new(db.pg_pool().clone()));
    let vehicle_svc = Arc::new(VehicleService::new(Arc::clone(&vehicle_repo), Arc::clone(&audit_svc)));

    let trip_repo = Arc::new(PgTripRepository::new(db.pg_pool().clone()));
    let trip_svc = Arc::new(TripService::new(trip_repo));

    let finance_repo = Arc::new(PgFinanceRepository::new(db.pg_pool().clone()));
    let finance_svc = Arc::new(FinanceService::new(finance_repo));

    // ── Clone for move into closure ───────────────────────────────────────────
    let config_data = web::Data::new((*config).clone());
    let db_data = web::Data::new(db);
    let auth_svc_data = web::Data::new(Arc::clone(&auth_svc));
    let auth_repo_data = web::Data::new(Arc::clone(&auth_repo));
    let driver_svc_data = web::Data::new(Arc::clone(&driver_svc));
    let vehicle_svc_data = web::Data::new(Arc::clone(&vehicle_svc));
    let trip_svc_data = web::Data::new(Arc::clone(&trip_svc));
    let finance_svc_data = web::Data::new(Arc::clone(&finance_svc));

    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allowed_origin(&config_data.frontend_url)
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .app_data(config_data.clone())
            .app_data(db_data.clone())
            .app_data(auth_svc_data.clone())
            .app_data(auth_repo_data.clone())
            .app_data(driver_svc_data.clone())
            .app_data(vehicle_svc_data.clone())
            .app_data(trip_svc_data.clone())
            .app_data(finance_svc_data.clone())
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api/v1")
                    .configure(auth::routes)
                    .configure(driver::routes)
                    .configure(vehicle::routes)
                    .configure(trip::routes)
                    .configure(finance::routes)
                    .configure(advance::routes)
                    .configure(salary::routes)
                    .configure(hr::routes)
                    .configure(invoice::routes)
                    .configure(report::routes)
                    .configure(settings::routes)
                    .configure(audit::routes)
                    .configure(uber::routes)
            )
    })
    .bind(&addr)?
    .run()
    .await?;

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
