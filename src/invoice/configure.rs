use std::sync::Arc;

use actix_web::web;
use sqlx::PgPool;

use crate::common::deps::SharedDeps;
use crate::invoice::{
    infrastructure::{PgInvoiceRepository, PdfService},
    application::service::InvoiceService,
    presentation::routes,
};

pub struct InvoiceDeps {
    pub svc: web::Data<Arc<InvoiceService>>,
}

impl Clone for InvoiceDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

/// Build invoice dependencies. Queries settings from DB for company info.
pub async fn build(deps: &SharedDeps) -> anyhow::Result<InvoiceDeps> {
    let repo = Arc::new(PgInvoiceRepository::new(deps.pool.clone()));
    let pdf = Arc::new(PdfService::new(&deps.config));

    let company_name = fetch_setting(&deps.pool, "company_name")
        .await
        .unwrap_or_else(|| "Fleet Management Co.".to_string());
    let company_address = fetch_setting(&deps.pool, "company_address")
        .await
        .unwrap_or_else(|| "Dubai, UAE".to_string());

    let svc = Arc::new(InvoiceService::new(
        repo,
        pdf,
        Arc::clone(&deps.audit),
        company_name,
        company_address,
    ));

    Ok(InvoiceDeps { svc: web::Data::new(svc) })
}

pub fn register(d: &InvoiceDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}

async fn fetch_setting(pool: &PgPool, key: &str) -> Option<String> {
    sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}
