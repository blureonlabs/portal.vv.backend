use async_trait::async_trait;
use crate::common::error::AppError;
use super::entity::Platform;

#[async_trait]
pub trait PlatformRepository: Send + Sync {
    async fn list_active(&self) -> Result<Vec<Platform>, AppError>;
}
