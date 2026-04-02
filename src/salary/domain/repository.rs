use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{CreateSalary, Salary};

#[async_trait]
pub trait SalaryRepository: Send + Sync {
    async fn list(&self, driver_id: Option<Uuid>, month: Option<NaiveDate>) -> Result<Vec<Salary>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Salary, AppError>;
    async fn find_by_driver_month(&self, driver_id: Uuid, period_month: NaiveDate) -> Result<Option<Salary>, AppError>;
    async fn upsert(&self, payload: CreateSalary) -> Result<Salary, AppError>;
}
