use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::settings::{
    domain::repository::SettingsRepository,
    infrastructure::PgSettingsRepository,
    application::service::SettingsService,
    presentation::routes,
};

pub struct SettingsDeps {
    pub svc: web::Data<Arc<SettingsService>>,
    /// Exposed so salary can use it.
    pub repo: Arc<dyn SettingsRepository>,
}

impl Clone for SettingsDeps {
    fn clone(&self) -> Self {
        Self {
            svc: self.svc.clone(),
            repo: Arc::clone(&self.repo),
        }
    }
}

pub fn build(deps: &SharedDeps) -> SettingsDeps {
    let repo: Arc<dyn SettingsRepository> =
        Arc::new(PgSettingsRepository::new(deps.pool.clone()));
    let svc = Arc::new(SettingsService::new(Arc::clone(&repo), Arc::clone(&deps.audit)));
    SettingsDeps {
        svc: web::Data::new(svc),
        repo,
    }
}

pub fn register(d: &SettingsDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
