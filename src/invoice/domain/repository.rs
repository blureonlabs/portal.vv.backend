use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{CreateInvoice, Invoice};

#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    async fn list(&self, driver_id: Option<Uuid>) -> Result<Vec<Invoice>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Invoice, AppError>;
    async fn create(&self, payload: CreateInvoice) -> Result<Invoice, AppError>;
    /// Returns the next sequential number for invoices in the given period month.
    async fn next_sequence(&self, period_start: NaiveDate) -> Result<u32, AppError>;
}
