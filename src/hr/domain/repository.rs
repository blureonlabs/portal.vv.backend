use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{ActionLeaveRequest, CreateLeaveRequest, LeaveRequest, LeaveStatus, LeaveType};

#[async_trait]
pub trait HrRepository: Send + Sync {
    async fn list(
        &self,
        driver_id: Option<Uuid>,
        status: Option<LeaveStatus>,
        leave_type: Option<LeaveType>,
    ) -> Result<Vec<LeaveRequest>, AppError>;

    async fn find_by_id(&self, id: Uuid) -> Result<LeaveRequest, AppError>;

    async fn create(&self, payload: CreateLeaveRequest) -> Result<LeaveRequest, AppError>;

    async fn approve(&self, payload: ActionLeaveRequest) -> Result<LeaveRequest, AppError>;

    async fn reject(&self, payload: ActionLeaveRequest) -> Result<LeaveRequest, AppError>;

    /// Count overlapping approved leaves for overlap validation.
    async fn count_overlapping_approved(
        &self,
        driver_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
        exclude_id: Option<Uuid>,
    ) -> Result<i64, AppError>;
}
