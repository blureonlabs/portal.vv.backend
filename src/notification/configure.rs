use std::sync::Arc;

use actix_web::web;

use crate::notification::application::service::NotificationService;

pub struct NotificationDeps {
    pub svc: web::Data<Arc<NotificationService>>,
}

impl Clone for NotificationDeps {
    fn clone(&self) -> Self {
        Self { svc: self.svc.clone() }
    }
}

/// NotificationService is constructed in main (it's a shared dep). This just wraps it.
pub fn build(notification_svc: Arc<NotificationService>) -> NotificationDeps {
    NotificationDeps { svc: web::Data::new(notification_svc) }
}

pub fn register(d: &NotificationDeps, cfg: &mut web::ServiceConfig) {
    cfg.app_data(d.svc.clone());
    // notification has no routes yet (TODO sprint)
}
