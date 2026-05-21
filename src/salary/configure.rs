use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::common::ports::DeductionPort;
use crate::salary::{
    infrastructure::postgres::PgSalaryRepository,
    infrastructure::pdf::SalaryPdfService,
    application::service::SalaryService,
    presentation::routes,
};
use crate::settings::domain::repository::SettingsRepository;

pub struct SalaryDeps {
    pub svc: web::Data<Arc<SalaryService>>,
}

impl Clone for SalaryDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

/// Build salary dependencies. Requires cross-feature deps from advance and settings.
pub fn build(
    deps: &SharedDeps,
    deduction_port: Arc<dyn DeductionPort>,
    settings_repo: Arc<dyn SettingsRepository>,
) -> SalaryDeps {
    let repo = Arc::new(PgSalaryRepository::new(deps.pool.clone()));
    let pdf = Arc::new(SalaryPdfService::new(&deps.config));
    let svc = Arc::new(SalaryService::new(
        repo,
        settings_repo,
        deduction_port,
        Arc::clone(&deps.audit),
        pdf,
    ));
    SalaryDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &SalaryDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
