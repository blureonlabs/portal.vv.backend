use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::finance::{
    infrastructure::PgFinanceRepository,
    application::service::FinanceService,
    presentation::routes,
};

pub struct FinanceDeps {
    pub svc: web::Data<Arc<FinanceService>>,
}

impl Clone for FinanceDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> FinanceDeps {
    let repo = Arc::new(PgFinanceRepository::new(deps.pool.clone()));
    let svc = Arc::new(FinanceService::new(repo, Arc::clone(&deps.audit)));
    FinanceDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &FinanceDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
