use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::trip::{
    infrastructure::PgTripRepository,
    application::service::TripService,
    presentation::routes,
};

pub struct TripDeps {
    pub svc: web::Data<Arc<TripService>>,
}

impl Clone for TripDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> TripDeps {
    let repo = Arc::new(PgTripRepository::new(deps.pool.clone()));
    let svc = Arc::new(TripService::new(repo, Arc::clone(&deps.audit)));
    TripDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &TripDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
