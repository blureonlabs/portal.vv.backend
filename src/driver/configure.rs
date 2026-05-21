use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::driver::{
    domain::repository::DriverRepository,
    infrastructure::PgDriverRepository,
    application::service::DriverService,
    presentation::routes,
};

pub struct DriverDeps {
    pub svc: web::Data<Arc<DriverService>>,
}

impl Clone for DriverDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> DriverDeps {
    let repo: Arc<dyn DriverRepository> =
        Arc::new(PgDriverRepository::new(deps.pool.clone()));
    let svc = Arc::new(DriverService::new(repo, Arc::clone(&deps.audit)));
    DriverDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &DriverDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
