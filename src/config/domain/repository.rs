use async_trait::async_trait;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{ConfigItem, ConfigDocumentType};

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn list_expense_categories(&self, active_only: bool) -> Result<Vec<ConfigItem>, AppError>;
    async fn list_leave_types(&self, active_only: bool) -> Result<Vec<ConfigItem>, AppError>;
    async fn list_document_types(&self, active_only: bool) -> Result<Vec<ConfigDocumentType>, AppError>;

    async fn create_expense_category(&self, name: &str, code: &str) -> Result<ConfigItem, AppError>;
    async fn create_leave_type(&self, name: &str, code: &str) -> Result<ConfigItem, AppError>;
    async fn create_document_type(&self, name: &str, code: &str, applies_to: &str) -> Result<ConfigDocumentType, AppError>;

    async fn update_config_item(&self, table: &str, id: Uuid, name: &str, is_active: bool, sort_order: i32) -> Result<(), AppError>;
    async fn delete_config_item(&self, table: &str, id: Uuid) -> Result<(), AppError>;
}
