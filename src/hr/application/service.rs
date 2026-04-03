use std::sync::Arc;

use chrono::{Duration, Local, NaiveDate};
use uuid::Uuid;

use crate::audit::application::service::AuditService;
use crate::common::{error::AppError, types::Role};
use crate::hr::domain::{
    entity::{ActionLeaveRequest, CreateLeaveRequest, LeaveRequest, LeaveStatus, LeaveType},
    repository::HrRepository,
};

pub struct HrService {
    repo: Arc<dyn HrRepository>,
    audit: Arc<AuditService>,
}

impl HrService {
    pub fn new(repo: Arc<dyn HrRepository>, audit: Arc<AuditService>) -> Self {
        Self { repo, audit }
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
        actor_id: Uuid,
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

        let today = Local::now().date_naive();
        if from_date < today - Duration::days(3) {
            return Err(AppError::BadRequest(
                "Cannot submit leave for dates more than 3 days in the past".into(),
            ));
        }

        let target_driver_id = match actor_role {
            Role::Driver => actor_driver_id
                .ok_or_else(|| AppError::Forbidden("Driver record not found".into()))?,
            Role::SuperAdmin | Role::Accountant | Role::Hr | Role::Owner => driver_id
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

        let request = self.repo
            .create(CreateLeaveRequest {
                driver_id: target_driver_id,
                r#type: leave_type,
                from_date,
                to_date,
                reason,
            })
            .await?;

        self.audit.log(actor_id, actor_role, "leave", Some(request.id), "leave.submitted",
            Some(serde_json::json!({ "driver_id": target_driver_id, "from_date": from_date, "to_date": to_date }))).await?;

        Ok(request)
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

        let leave = self.repo
            .approve(ActionLeaveRequest { id: request_id, actioned_by: actor_id, rejection_reason: None })
            .await?;

        self.audit.log(actor_id, actor_role, "leave", Some(request_id), "leave.approved",
            Some(serde_json::json!({ "driver_id": leave.driver_id }))).await?;

        Ok(leave)
    }

    pub async fn bulk_approve(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        request_ids: Vec<Uuid>,
    ) -> Result<u64, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Hr => {}
            _ => return Err(AppError::Forbidden("Only super_admin or hr can bulk-approve leave".into())),
        }

        if request_ids.is_empty() {
            return Ok(0);
        }

        let approved_count = self.repo.bulk_approve(&request_ids, actor_id).await?;

        self.audit.log(actor_id, actor_role, "leave", None, "leave.bulk_approved",
            Some(serde_json::json!({
                "request_ids": request_ids,
                "approved_count": approved_count
            }))).await?;

        Ok(approved_count)
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
        let leave = self.repo
            .reject(ActionLeaveRequest {
                id: request_id,
                actioned_by: actor_id,
                rejection_reason: Some(rejection_reason.clone()),
            })
            .await?;

        self.audit.log(actor_id, actor_role, "leave", Some(request_id), "leave.rejected",
            Some(serde_json::json!({ "driver_id": leave.driver_id, "reason": rejection_reason }))).await?;

        Ok(leave)
    }
}
