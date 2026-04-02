use anyhow::Context;

#[derive(Debug, Clone)]
pub struct AppConfig {
    // Server
    pub host: String,
    pub port: u16,
    pub frontend_url: String,

    // Database
    pub database_url: String,

    // Supabase
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub supabase_service_role_key: String,
    pub supabase_jwt_secret: String,
    pub supabase_storage_bucket: String,

    // Email
    pub resend_api_key: String,
    pub resend_from_email: String,

    // Security
    pub seed_key: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("PORT must be a valid number")?,
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),

            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL is required")?,

            supabase_url: std::env::var("SUPABASE_URL")
                .context("SUPABASE_URL is required")?,
            supabase_anon_key: std::env::var("SUPABASE_ANON_KEY")
                .context("SUPABASE_ANON_KEY is required")?,
            supabase_service_role_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                .context("SUPABASE_SERVICE_ROLE_KEY is required")?,
            supabase_jwt_secret: std::env::var("SUPABASE_JWT_SECRET")
                .context("SUPABASE_JWT_SECRET is required")?,
            supabase_storage_bucket: std::env::var("SUPABASE_STORAGE_BUCKET")
                .unwrap_or_else(|_| "fms-files".to_string()),

            resend_api_key: std::env::var("RESEND_API_KEY")
                .context("RESEND_API_KEY is required")?,
            resend_from_email: std::env::var("RESEND_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@fleetms.ae".to_string()),

            seed_key: std::env::var("SEED_KEY").unwrap_or_default(),
        })
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
