use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::report::{
    application::service::ReportService,
    presentation::routes,
};

pub struct ReportDeps {
    pub svc: web::Data<Arc<ReportService>>,
}

impl Clone for ReportDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> ReportDeps {
    let svc = Arc::new(ReportService::new(deps.pool.clone()));
    ReportDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &ReportDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
