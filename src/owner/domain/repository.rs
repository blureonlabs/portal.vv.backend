use async_trait::async_trait;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::Owner;

#[async_trait]
pub trait OwnerRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Owner>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, AppError>;
    async fn find_by_profile_id(&self, profile_id: Uuid) -> Result<Option<Owner>, AppError>;
    async fn create(&self, profile_id: Uuid, company_name: Option<&str>, notes: Option<&str>) -> Result<Owner, AppError>;
    async fn update(&self, id: Uuid, company_name: Option<&str>, notes: Option<&str>) -> Result<Owner, AppError>;
}
