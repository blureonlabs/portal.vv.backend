use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::{error::AppError, types::Role};
use crate::advance::domain::{
    entity::{Advance, AdvanceStatus, ApproveAdvance, CreateAdvance, PayAdvance, PaymentMethod, RejectAdvance},
    repository::AdvanceRepository,
};

pub struct AdvanceService {
    repo: Arc<dyn AdvanceRepository>,
}

impl AdvanceService {
    pub fn new(repo: Arc<dyn AdvanceRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        status: Option<AdvanceStatus>,
    ) -> Result<Vec<Advance>, AppError> {
        // Drivers see only their own advances
        let effective_driver_id = if *actor_role == Role::Driver {
            actor_driver_id
        } else {
            driver_id
        };
        self.repo.list(effective_driver_id, status).await
    }

    pub async fn request(
        &self,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        amount_aed: Decimal,
        reason: String,
    ) -> Result<Advance, AppError> {
        // Resolve who the advance is for
        let target_driver_id = match actor_role {
            Role::Driver => actor_driver_id
                .ok_or_else(|| AppError::Forbidden("Driver record not found".into()))?,
            Role::SuperAdmin | Role::Accountant | Role::Hr => driver_id
                .ok_or_else(|| AppError::BadRequest("driver_id is required".into()))?,
        };

        // Enforce 1-pending limit
        let pending_count = self.repo.count_pending(target_driver_id).await?;
        if pending_count > 0 {
            return Err(AppError::BadRequest(
                "Driver already has a pending advance request".into(),
            ));
        }

        self.repo
            .create(CreateAdvance {
                driver_id: target_driver_id,
                amount_aed,
                reason,
            })
            .await
    }

    pub async fn approve(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        advance_id: Uuid,
    ) -> Result<Advance, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Accountant => {}
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can approve advances".into())),
        }
        self.repo.approve(ApproveAdvance { id: advance_id, actioned_by: actor_id }).await
    }

    pub async fn reject(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        advance_id: Uuid,
        rejection_reason: String,
    ) -> Result<Advance, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Accountant => {}
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can reject advances".into())),
        }
        self.repo
            .reject(RejectAdvance {
                id: advance_id,
                actioned_by: actor_id,
                rejection_reason,
            })
            .await
    }

    pub async fn pay(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        advance_id: Uuid,
        payment_date: NaiveDate,
        method: PaymentMethod,
        salary_period: Option<NaiveDate>,
    ) -> Result<Advance, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Accountant => {}
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can mark advances paid".into())),
        }
        self.repo
            .pay(PayAdvance {
                id: advance_id,
                actioned_by: actor_id,
                payment_date,
                method,
                salary_period,
            })
            .await
    }
}
