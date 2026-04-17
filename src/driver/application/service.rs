use std::sync::Arc;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::audit::application::service::AuditService;
use crate::common::{error::AppError, types::{Role, SalaryType}};
use crate::driver::domain::{entity::Driver, repository::DriverRepository};

pub struct DriverService {
    pub repo: Arc<dyn DriverRepository>,
    pub audit: Arc<AuditService>,
}

impl DriverService {
    pub fn new(repo: Arc<dyn DriverRepository>, audit: Arc<AuditService>) -> Self {
        Self { repo, audit }
    }

    pub async fn list(&self) -> Result<Vec<Driver>, AppError> {
        self.repo.list().await
    }

    pub async fn get(&self, id: Uuid) -> Result<Driver, AppError> {
        self.repo.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound(format!("Driver {} not found", id)))
    }

    pub async fn create(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        profile_id: Uuid,
        nationality: String,
        salary_type: SalaryType,
        room_rent_aed: Decimal,
        commission_rate: Option<Decimal>,
        joining_date: Option<NaiveDate>,
    ) -> Result<Driver, AppError> {
        if self.repo.find_by_profile_id(profile_id).await?.is_some() {
            return Err(AppError::Conflict("Driver profile already exists for this user".into()));
        }

        let driver = self.repo.create(profile_id, &nationality, salary_type, room_rent_aed, commission_rate, joining_date).await?;

        self.audit.log(actor_id, actor_role, "driver", Some(driver.id), "created",
            Some(serde_json::json!({ "profile_id": profile_id }))).await?;

        Ok(driver)
    }

    pub async fn update(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        id: Uuid,
        nationality: String,
        salary_type: SalaryType,
        room_rent_aed: Decimal,
        commission_rate: Option<Decimal>,
        joining_date: Option<NaiveDate>,
    ) -> Result<Driver, AppError> {
        let old = self.repo.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Driver not found".into()))?;

        // Log changed fields to driver_edits (append-only)
        if old.nationality != nationality {
            self.repo.log_edit(id, actor_id, "nationality", Some(&old.nationality), Some(&nationality)).await?;
        }
        let old_st = format!("{:?}", old.salary_type);
        let new_st = format!("{:?}", salary_type);
        if old_st != new_st {
            self.repo.log_edit(id, actor_id, "salary_type", Some(&old_st), Some(&new_st)).await?;
        }
        if old.room_rent_aed != room_rent_aed {
            self.repo.log_edit(id, actor_id, "room_rent_aed", Some(&old.room_rent_aed.to_string()), Some(&room_rent_aed.to_string())).await?;
        }
        let old_cr = old.commission_rate.map(|r| r.to_string());
        let new_cr = commission_rate.map(|r| r.to_string());
        if old_cr != new_cr {
            self.repo.log_edit(id, actor_id, "commission_rate", old_cr.as_deref(), new_cr.as_deref()).await?;
        }
        let old_jd = old.joining_date.map(|d| d.to_string());
        let new_jd = joining_date.map(|d| d.to_string());
        if old_jd != new_jd {
            self.repo.log_edit(id, actor_id, "joining_date", old_jd.as_deref(), new_jd.as_deref()).await?;
        }

        let driver = self.repo.update(id, &nationality, salary_type, room_rent_aed, commission_rate, joining_date).await?;

        self.audit.log(actor_id, actor_role, "driver", Some(id), "updated", None).await?;

        Ok(driver)
    }

    pub async fn deactivate(&self, actor_id: Uuid, actor_role: &Role, id: Uuid) -> Result<(), AppError> {
        let driver = self.repo.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Driver not found".into()))?;

        if !driver.is_active {
            return Err(AppError::BadRequest("Driver is already inactive".into()));
        }

        if self.repo.has_active_vehicle(id).await? {
            return Err(AppError::BadRequest(
                "Cannot deactivate driver: they have an active vehicle assigned. Unassign first.".into()
            ));
        }

        self.repo.set_active(id, false).await?;
        self.audit.log(actor_id, actor_role, "driver", Some(id), "deactivated", None).await?;
        Ok(())
    }

    pub async fn activate(&self, actor_id: Uuid, actor_role: &Role, id: Uuid) -> Result<(), AppError> {
        self.repo.find_by_id(id).await?
            .ok_or_else(|| AppError::NotFound("Driver not found".into()))?;

        self.repo.set_active(id, true).await?;
        self.audit.log(actor_id, actor_role, "driver", Some(id), "activated", None).await?;
        Ok(())
    }
}
