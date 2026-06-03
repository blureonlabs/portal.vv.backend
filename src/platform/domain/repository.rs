use async_trait::async_trait;
use uuid::Uuid;
use crate::common::error::AppError;
use super::entity::{Platform, CreatePlatform, UpdatePlatform};

#[async_trait]
pub trait PlatformRepository: Send + Sync {
    async fn list_active(&self) -> Result<Vec<Platform>, AppError>;
    async fn create(&self, payload: CreatePlatform) -> Result<Platform, AppError>;
    async fn update(&self, id: Uuid, payload: UpdatePlatform) -> Result<Platform, AppError>;
    async fn deactivate(&self, id: Uuid) -> Result<(), AppError>;
}
