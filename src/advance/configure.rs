use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::advance::{
    domain::repository::AdvanceRepository,
    infrastructure::PgAdvanceRepository,
    application::service::AdvanceService,
    presentation::routes,
};

pub struct AdvanceDeps {
    pub svc: web::Data<Arc<AdvanceService>>,
    /// Exposed so salary can use it as DeductionPort.
    pub repo: Arc<PgAdvanceRepository>,
}

impl Clone for AdvanceDeps {
    fn clone(&self) -> Self {
        Self {
            svc: self.svc.clone(),
            repo: Arc::clone(&self.repo),
        }
    }
}

pub fn build(deps: &SharedDeps) -> AdvanceDeps {
    let repo = Arc::new(PgAdvanceRepository::new(deps.pool.clone()));
    let svc = Arc::new(AdvanceService::new(
        Arc::clone(&repo) as Arc<dyn AdvanceRepository>,
        Arc::clone(&deps.audit),
    ));
    AdvanceDeps {
        svc: web::Data::new(svc),
        repo,
    }
}

pub fn register(d: &AdvanceDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
