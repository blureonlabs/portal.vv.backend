use std::sync::Arc;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::common::{error::AppError, types::Role};
use crate::hr::domain::{
    entity::{ActionLeaveRequest, CreateLeaveRequest, LeaveRequest, LeaveStatus, LeaveType},
    repository::HrRepository,
};

pub struct HrService {
    repo: Arc<dyn HrRepository>,
}

impl HrService {
    pub fn new(repo: Arc<dyn HrRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        status: Option<LeaveStatus>,
        leave_type: Option<LeaveType>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        let effective_driver_id = if *actor_role == Role::Driver {
            actor_driver_id
        } else {
            driver_id
        };
        self.repo.list(effective_driver_id, status, leave_type).await
    }

    pub async fn submit(
        &self,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        leave_type: LeaveType,
        from_date: NaiveDate,
        to_date: NaiveDate,
        reason: String,
    ) -> Result<LeaveRequest, AppError> {
        if from_date > to_date {
            return Err(AppError::BadRequest("from_date must be on or before to_date".into()));
        }

        let target_driver_id = match actor_role {
            Role::Driver => actor_driver_id
                .ok_or_else(|| AppError::Forbidden("Driver record not found".into()))?,
            Role::SuperAdmin | Role::Accountant | Role::Hr => driver_id
                .ok_or_else(|| AppError::BadRequest("driver_id is required".into()))?,
        };

        // Overlap check: reject if an approved leave already covers these dates
        let overlaps = self.repo
            .count_overlapping_approved(target_driver_id, from_date, to_date, None)
            .await?;
        if overlaps > 0 {
            return Err(AppError::BadRequest(
                "Driver already has an approved leave overlapping these dates".into(),
            ));
        }

        self.repo
            .create(CreateLeaveRequest {
                driver_id: target_driver_id,
                r#type: leave_type,
                from_date,
                to_date,
                reason,
            })
            .await
    }

    pub async fn approve(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        request_id: Uuid,
    ) -> Result<LeaveRequest, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Hr => {}
            _ => return Err(AppError::Forbidden("Only super_admin or hr can approve leave".into())),
        }

        // Fetch request to get driver_id + dates for overlap check before approving
        let req = self.repo.find_by_id(request_id).await?;
        let overlaps = self.repo
            .count_overlapping_approved(req.driver_id, req.from_date, req.to_date, Some(req.id))
            .await?;
        if overlaps > 0 {
            return Err(AppError::BadRequest(
                "Approving this request would overlap with an existing approved leave".into(),
            ));
        }

        self.repo
            .approve(ActionLeaveRequest { id: request_id, actioned_by: actor_id, rejection_reason: None })
            .await
    }

    pub async fn reject(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        request_id: Uuid,
        rejection_reason: String,
    ) -> Result<LeaveRequest, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Hr => {}
            _ => return Err(AppError::Forbidden("Only super_admin or hr can reject leave".into())),
        }
        self.repo
            .reject(ActionLeaveRequest {
                id: request_id,
                actioned_by: actor_id,
                rejection_reason: Some(rejection_reason),
            })
            .await
    }
}
