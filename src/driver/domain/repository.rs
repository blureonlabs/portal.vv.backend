use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::{error::AppError, types::SalaryType};
use super::entity::{Driver, DriverEdit};

#[async_trait]
pub trait DriverRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Driver>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Driver>, AppError>;
    async fn find_by_profile_id(&self, profile_id: Uuid) -> Result<Option<Driver>, AppError>;
    async fn create(&self, profile_id: Uuid, nationality: &str, salary_type: SalaryType, room_rent_aed: Decimal, commission_rate: Option<Decimal>, joining_date: Option<NaiveDate>) -> Result<Driver, AppError>;
    async fn update(&self, id: Uuid, nationality: &str, salary_type: SalaryType, room_rent_aed: Decimal, commission_rate: Option<Decimal>, joining_date: Option<NaiveDate>) -> Result<Driver, AppError>;
    async fn set_active(&self, id: Uuid, active: bool) -> Result<(), AppError>;
    async fn has_active_vehicle(&self, id: Uuid) -> Result<bool, AppError>;
    async fn log_edit(&self, driver_id: Uuid, changed_by: Uuid, field: &str, old_val: Option<&str>, new_val: Option<&str>) -> Result<(), AppError>;
    async fn list_edits(&self, driver_id: Uuid) -> Result<Vec<DriverEdit>, AppError>;
    async fn set_self_entry(&self, id: Uuid, enabled: bool) -> Result<(), AppError>;
}
