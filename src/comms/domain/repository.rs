use async_trait::async_trait;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{Broadcast, BroadcastStatus};

#[async_trait]
pub trait BroadcastRepository: Send + Sync {
    async fn create(&self, broadcast: &Broadcast) -> Result<Broadcast, AppError>;
    async fn list(&self) -> Result<Vec<Broadcast>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Broadcast>, AppError>;
    async fn update_status(&self, id: Uuid, status: BroadcastStatus, recipient_count: i32) -> Result<(), AppError>;
}
