use std::sync::Arc;

use actix_web::web;

use crate::common::deps::SharedDeps;
use crate::document::{
    domain::repository::DocumentRepository,
    infrastructure::PgDocumentRepository,
    application::service::DocumentService,
    presentation::routes,
};

pub struct DocumentDeps {
    pub svc: web::Data<Arc<DocumentService>>,
}

impl Clone for DocumentDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

pub fn build(deps: &SharedDeps) -> DocumentDeps {
    let repo: Arc<dyn DocumentRepository> =
        Arc::new(PgDocumentRepository::new(deps.pool.clone()));
    let svc = Arc::new(DocumentService::new(repo));
    DocumentDeps { svc: web::Data::new(svc) }
}

pub fn register(d: &DocumentDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    routes(cfg);
}
