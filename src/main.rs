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

    // ── Clone for move into closure ───────────────────────────────────────────
    let config_data = web::Data::new((*config).clone());
    let db_data = web::Data::new(db);
    let auth_svc_data = web::Data::new(Arc::clone(&auth_svc));
    let auth_repo_data = web::Data::new(Arc::clone(&auth_repo));

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
