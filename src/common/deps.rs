use std::sync::Arc;

use sqlx::PgPool;

use crate::audit::application::service::AuditService;
use crate::config::AppConfig;
use crate::notification::application::service::NotificationService;

/// Shared dependencies injected into each feature's configure function.
pub struct SharedDeps {
    pub pool: PgPool,
    pub config: Arc<AppConfig>,
    pub audit: Arc<AuditService>,
    pub notification: Arc<NotificationService>,
}
