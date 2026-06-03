use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::config::{
    domain::repository::ConfigRepository,
    infrastructure::postgres::PgConfigRepository,
    application::service::ConfigService,
    presentation::routes,
};

pub struct ConfigDeps {
    pub svc: web::Data<Arc<ConfigService>>,
}

impl Clone for ConfigDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> ConfigDeps {
    let repo: Arc<dyn ConfigRepository> =
        Arc::new(PgConfigRepository::new(deps.pool.clone()));
    let svc = Arc::new(ConfigService::new(repo, Arc::clone(&deps.audit)));
    ConfigDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &ConfigDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
