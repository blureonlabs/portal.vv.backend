use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{CreateTrip, Trip};

#[async_trait]
pub trait TripRepository: Send + Sync {
    async fn list(
        &self,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Trip>, AppError>;

    #[allow(dead_code)]
    async fn find_by_id(&self, id: Uuid) -> Result<Trip, AppError>;

    async fn create(&self, payload: CreateTrip) -> Result<Trip, AppError>;

    async fn soft_delete(&self, id: Uuid) -> Result<(), AppError>;

    /// Sum of cash+card+other for a driver on a specific date (non-deleted rows).
    async fn daily_total(&self, driver_id: Uuid, date: NaiveDate) -> Result<Decimal, AppError>;

    /// Bulk insert — used for CSV import.
    async fn bulk_insert(&self, rows: Vec<CreateTrip>) -> Result<Vec<Trip>, AppError>;

    /// Fetch trip_cap_aed from settings table.
    async fn get_trip_cap(&self) -> Result<Decimal, AppError>;

    /// Check if driver has self_entry_enabled.
    async fn driver_self_entry_enabled(&self, driver_id: Uuid) -> Result<bool, AppError>;

    /// Resolve driver record id from profile (auth user) id.
    async fn find_driver_id_by_profile(&self, profile_id: Uuid) -> Result<Option<Uuid>, AppError>;

    /// Find non-deleted trips for a driver on a specific date.
    async fn find_by_driver_and_date(&self, driver_id: Uuid, date: NaiveDate) -> Result<Vec<Trip>, AppError>;
}
