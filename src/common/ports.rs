use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::error::AppError;

/// Cross-feature contract: salary module consumes this to get advance deductions.
/// advance::infrastructure implements this — salary never imports advance directly.
#[async_trait]
pub trait DeductionPort: Send + Sync {
    /// Returns total advance amount deducted for a driver in a given salary period.
    async fn get_advance_deductions(
        &self,
        driver_id: Uuid,
        period_month: NaiveDate,
    ) -> Result<Decimal, AppError>;

    /// Returns advance IDs linked to this salary period (stored in deductions_json).
    async fn get_advance_ids_for_period(
        &self,
        driver_id: Uuid,
        period_month: NaiveDate,
    ) -> Result<Vec<Uuid>, AppError>;
}
