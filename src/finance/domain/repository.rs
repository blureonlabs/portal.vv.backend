use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{CashHandover, CreateExpense, CreateHandover, Expense};

#[async_trait]
pub trait FinanceRepository: Send + Sync {
    async fn list_expenses(
        &self,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Expense>, i64), AppError>;

    async fn create_expense(&self, payload: CreateExpense) -> Result<Expense, AppError>;

    async fn list_handovers(
        &self,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<CashHandover>, i64), AppError>;

    async fn create_handover(&self, payload: CreateHandover) -> Result<CashHandover, AppError>;
}
