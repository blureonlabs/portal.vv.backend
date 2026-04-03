use std::sync::Arc;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::audit::application::service::AuditService;
use crate::common::{error::AppError, types::Role};
use crate::vehicle::domain::{
    entity::{Vehicle, VehicleAssignment, VehicleServiceRecord, VehicleStatus},
    repository::VehicleRepository,
};

pub struct VehicleService {
    pub repo: Arc<dyn VehicleRepository>,
    pub audit: Arc<AuditService>,
}

impl VehicleService {
    pub fn new(repo: Arc<dyn VehicleRepository>, audit: Arc<AuditService>) -> Self {
        Self { repo, audit }
    }

    pub async fn list(&self) -> Result<Vec<Vehicle>, AppError> {
        self.repo.list().await
    }

    pub async fn get(&self, id: Uuid) -> Result<Vehicle, AppError> {
        self.repo.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound(format!("Vehicle {} not found", id)))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        plate_number: String,
        make: String,
        model: String,
        year: i32,
        color: Option<String>,
        registration_date: Option<NaiveDate>,
        registration_expiry: Option<NaiveDate>,
        insurance_expiry: Option<NaiveDate>,
        owner_id: Option<Uuid>,
    ) -> Result<Vehicle, AppError> {
        let vehicle = self.repo.create(
            &plate_number, &make, &model, year,
            color.as_deref(),
            registration_date, registration_expiry, insurance_expiry, owner_id,
        ).await?;

        self.audit.log(actor_id, actor_role, "vehicle", Some(vehicle.id), "created",
            Some(serde_json::json!({ "plate": plate_number }))).await?;

        Ok(vehicle)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        id: Uuid,
        plate_number: String,
        make: String,
        model: String,
        year: i32,
        color: Option<String>,
        registration_date: Option<NaiveDate>,
        registration_expiry: Option<NaiveDate>,
        insurance_expiry: Option<NaiveDate>,
        owner_id: Option<Uuid>,
    ) -> Result<Vehicle, AppError> {
        self.repo.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Vehicle not found".into()))?;

        let vehicle = self.repo.update(
            id, &plate_number, &make, &model, year,
            color.as_deref(),
            registration_date, registration_expiry, insurance_expiry, owner_id,
        ).await?;

        self.audit.log(actor_id, actor_role, "vehicle", Some(id), "updated", None).await?;

        Ok(vehicle)
    }

    pub async fn assign_driver(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        vehicle_id: Uuid,
        driver_id: Uuid,
    ) -> Result<(), AppError> {
        let vehicle = self.repo.find_by_id(vehicle_id).await?
            .ok_or_else(|| AppError::NotFound("Vehicle not found".into()))?;

        if vehicle.status != VehicleStatus::Available {
            return Err(AppError::BadRequest("Vehicle is not available for assignment".into()));
        }

        if self.repo.driver_has_active_assignment(driver_id).await? {
            return Err(AppError::BadRequest("Driver already has a vehicle assigned".into()));
        }

        self.repo.assign(vehicle_id, driver_id, actor_id).await?;

        self.audit.log(actor_id, actor_role, "vehicle", Some(vehicle_id), "assigned",
            Some(serde_json::json!({ "driver_id": driver_id }))).await?;

        Ok(())
    }

    pub async fn unassign_driver(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        vehicle_id: Uuid,
    ) -> Result<(), AppError> {
        let vehicle = self.repo.find_by_id(vehicle_id).await?
            .ok_or_else(|| AppError::NotFound("Vehicle not found".into()))?;

        if vehicle.status != VehicleStatus::Assigned {
            return Err(AppError::BadRequest("Vehicle has no active assignment".into()));
        }

        self.repo.unassign(vehicle_id).await?;

        self.audit.log(actor_id, actor_role, "vehicle", Some(vehicle_id), "unassigned", None).await?;

        Ok(())
    }

    pub async fn add_service_record(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        vehicle_id: Uuid,
        service_date: NaiveDate,
        service_type: String,
        description: Option<String>,
        cost: Decimal,
        next_due: Option<NaiveDate>,
    ) -> Result<VehicleServiceRecord, AppError> {
        self.repo.find_by_id(vehicle_id).await?
            .ok_or_else(|| AppError::NotFound("Vehicle not found".into()))?;

        let record = self.repo.add_service_record(
            vehicle_id, service_date, &service_type,
            description.as_deref(), cost, next_due, actor_id,
        ).await?;

        self.audit.log(actor_id, actor_role, "vehicle_service", Some(record.id), "created", None).await?;

        Ok(record)
    }

    pub async fn list_assignments(&self, vehicle_id: Uuid) -> Result<Vec<VehicleAssignment>, AppError> {
        self.repo.list_assignments(vehicle_id).await
    }

    pub async fn list_service_records(&self, vehicle_id: Uuid) -> Result<Vec<VehicleServiceRecord>, AppError> {
        self.repo.list_service_records(vehicle_id).await
    }
}
