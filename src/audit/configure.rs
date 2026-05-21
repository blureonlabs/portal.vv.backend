use std::sync::Arc;

use actix_web::web;

use crate::audit::{
    application::service::AuditService,
    presentation::routes,
};

pub struct AuditDeps {
    pub svc: web::Data<Arc<AuditService>>,
}

impl Clone for AuditDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

/// AuditService is constructed in main (it's a shared dep). This just wraps it for registration.
pub fn build(audit_svc: Arc<AuditService>) -> AuditDeps {
    AuditDeps { svc: web::Data::new(audit_svc) }
}

pub fn register(d: &AuditDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
