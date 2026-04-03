use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::error::AppError;
use super::entity::{Vehicle, VehicleAssignment, VehicleServiceRecord};

#[async_trait]
pub trait VehicleRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Vehicle>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Vehicle>, AppError>;
    async fn create(
        &self,
        plate_number: &str,
        make: &str,
        model: &str,
        year: i32,
        color: Option<&str>,
        registration_date: Option<NaiveDate>,
        registration_expiry: Option<NaiveDate>,
        insurance_expiry: Option<NaiveDate>,
        owner_id: Option<Uuid>,
    ) -> Result<Vehicle, AppError>;
    async fn update(
        &self,
        id: Uuid,
        plate_number: &str,
        make: &str,
        model: &str,
        year: i32,
        color: Option<&str>,
        registration_date: Option<NaiveDate>,
        registration_expiry: Option<NaiveDate>,
        insurance_expiry: Option<NaiveDate>,
        owner_id: Option<Uuid>,
    ) -> Result<Vehicle, AppError>;
    async fn assign(&self, vehicle_id: Uuid, driver_id: Uuid, assigned_by: Uuid) -> Result<(), AppError>;
    async fn unassign(&self, vehicle_id: Uuid) -> Result<(), AppError>;
    async fn driver_has_active_assignment(&self, driver_id: Uuid) -> Result<bool, AppError>;
    async fn list_assignments(&self, vehicle_id: Uuid) -> Result<Vec<VehicleAssignment>, AppError>;
    async fn add_service_record(
        &self,
        vehicle_id: Uuid,
        service_date: NaiveDate,
        service_type: &str,
        description: Option<&str>,
        cost: Decimal,
        next_due: Option<NaiveDate>,
        logged_by: Uuid,
    ) -> Result<VehicleServiceRecord, AppError>;
    async fn list_service_records(&self, vehicle_id: Uuid) -> Result<Vec<VehicleServiceRecord>, AppError>;
}
