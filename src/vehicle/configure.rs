use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::vehicle::{
    domain::repository::VehicleRepository,
    infrastructure::PgVehicleRepository,
    application::service::VehicleService,
    presentation::routes,
};

pub struct VehicleDeps {
    pub svc: web::Data<Arc<VehicleService>>,
}

impl Clone for VehicleDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> VehicleDeps {
    let repo: Arc<dyn VehicleRepository> =
        Arc::new(PgVehicleRepository::new(deps.pool.clone()));
    let svc = Arc::new(VehicleService::new(repo, Arc::clone(&deps.audit)));
    VehicleDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &VehicleDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
