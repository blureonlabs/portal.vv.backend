use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::comms::{
    infrastructure::PgCommsRepository,
    application::service::CommsService,
    presentation::routes,
};

pub struct CommsDeps {
    pub svc: web::Data<Arc<CommsService>>,
}

impl Clone for CommsDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> CommsDeps {
    let repo = PgCommsRepository::new(deps.pool.clone());
    let svc = Arc::new(CommsService::new(repo, Arc::clone(&deps.notification)));
    CommsDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &CommsDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
