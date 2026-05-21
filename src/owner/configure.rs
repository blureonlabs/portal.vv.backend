use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::owner::{
    domain::repository::OwnerRepository,
    infrastructure::PgOwnerRepository,
    application::service::OwnerService,
    presentation::routes,
};

pub struct OwnerDeps {
    pub svc: web::Data<Arc<OwnerService>>,
}

impl Clone for OwnerDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> OwnerDeps {
    let repo: Arc<dyn OwnerRepository> =
        Arc::new(PgOwnerRepository::new(deps.pool.clone()));
    let svc = Arc::new(OwnerService::new(repo));
    OwnerDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &OwnerDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
