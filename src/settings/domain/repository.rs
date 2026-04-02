use async_trait::async_trait;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::Setting;

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Setting>, AppError>;
    #[allow(dead_code)]
    async fn get(&self, key: &str) -> Result<Option<Setting>, AppError>;
    async fn upsert(&self, key: &str, value: &str, updated_by: Uuid) -> Result<Setting, AppError>;
}
