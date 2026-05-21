use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::platform::{
    domain::repository::PlatformRepository,
    infrastructure::postgres::PgPlatformRepository,
    presentation::routes,
};

pub struct PlatformDeps {
    pub repo: web::Data<Arc<dyn PlatformRepository>>,
}

impl Clone for PlatformDeps {
    fn clone(&self) -> Self {
        Self { repo: self.repo.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> PlatformDeps {
    let repo: Arc<dyn PlatformRepository> =
        Arc::new(PgPlatformRepository::new(deps.pool.clone()));
    PlatformDeps { repo: web::Data::new(repo) }
}

pub fn register(d: &PlatformDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.repo.clone());
    routes(cfg);
}
