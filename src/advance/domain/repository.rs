use async_trait::async_trait;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{Advance, AdvanceStatus, ApproveAdvance, CreateAdvance, PayAdvance, RejectAdvance};

#[async_trait]
pub trait AdvanceRepository: Send + Sync {
    async fn list(&self, driver_id: Option<Uuid>, status: Option<AdvanceStatus>) -> Result<Vec<Advance>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Advance, AppError>;
    async fn create(&self, payload: CreateAdvance) -> Result<Advance, AppError>;
    async fn approve(&self, payload: ApproveAdvance) -> Result<Advance, AppError>;
    async fn reject(&self, payload: RejectAdvance) -> Result<Advance, AppError>;
    async fn pay(&self, payload: PayAdvance) -> Result<Advance, AppError>;
    async fn count_pending(&self, driver_id: Uuid) -> Result<i64, AppError>;
}
