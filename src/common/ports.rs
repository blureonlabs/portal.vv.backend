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

/// Cross-feature contract: trip module defines this; uber module implements it.
/// Allows plugging in Uber API without changing trip logic.
#[async_trait]
pub trait TripSourcePort: Send + Sync {
    async fn fetch_trips(
        &self,
        driver_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<ExternalTrip>, AppError>;
}

/// Minimal trip data returned by any TripSourcePort implementor.
pub struct ExternalTrip {
    pub driver_id: Uuid,
    pub trip_date: NaiveDate,
    pub cash_aed: Decimal,
    pub card_aed: Decimal,
    pub other_aed: Decimal,
    pub external_id: Option<String>,
    pub notes: Option<String>,
}
