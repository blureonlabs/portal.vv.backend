use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::hr::{
    infrastructure::PgHrRepository,
    application::service::HrService,
    presentation::routes,
};

pub struct HrDeps {
    pub svc: web::Data<Arc<HrService>>,
}

impl Clone for HrDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> HrDeps {
    let repo = Arc::new(PgHrRepository::new(deps.pool.clone()));
    let svc = Arc::new(HrService::new(repo, Arc::clone(&deps.audit)));
    HrDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &HrDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
