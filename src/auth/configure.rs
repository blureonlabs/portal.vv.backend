use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::auth::{
    domain::repository::AuthRepository,
    infrastructure::{PgAuthRepository, SupabaseAdminClient},
    application::service::AuthService,
    presentation::routes,
};

/// Constructed auth dependencies ready for registration in the HTTP server.
pub struct AuthDeps {
    pub auth_svc: web::Data<Arc<AuthService>>,
    pub auth_repo: web::Data<Arc<dyn AuthRepository>>,
    pub supabase: web::Data<Arc<SupabaseAdminClient>>,
}

impl Clone for AuthDeps {
    fn clone(&self) -> Self {
        Self {
            auth_svc: self.auth_svc.clone(),
            auth_repo: self.auth_repo.clone(),
            supabase: self.supabase.clone(),
        }
    }
}

/// Build auth dependencies (called once at startup).
pub fn build(deps: &SharedDeps) -> AuthDeps {
    let auth_repo: Arc<dyn AuthRepository> =
        Arc::new(PgAuthRepository::new(deps.pool.clone()));
    let supabase = Arc::new(SupabaseAdminClient::new(&deps.config));
    let auth_svc = Arc::new(AuthService::new(
        Arc::clone(&auth_repo),
        Arc::clone(&supabase),
        Arc::clone(&deps.config),
        Arc::clone(&deps.notification),
        Arc::clone(&deps.audit),
    ));

    AuthDeps {
        auth_svc: web::Data::new(auth_svc),
        auth_repo: web::Data::new(auth_repo),
        supabase: web::Data::new(supabase),
    }
}

/// Register auth app_data and routes onto the service config.
pub fn register(d: &AuthDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.auth_svc.clone());
    cfg.app_data(d.auth_repo.clone());
    cfg.app_data(d.supabase.clone());
    routes(cfg);
}
