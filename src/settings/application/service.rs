use std::sync::Arc;

use uuid::Uuid;

use crate::audit::application::service::AuditService;
use crate::common::{error::AppError, types::Role};
use crate::settings::domain::{
    entity::{Setting, ACCOUNTANT_ALLOWED_KEYS},
    repository::SettingsRepository,
};

pub struct SettingsService {
    repo: Arc<dyn SettingsRepository>,
    audit: Arc<AuditService>,
}

impl SettingsService {
    pub fn new(repo: Arc<dyn SettingsRepository>, audit: Arc<AuditService>) -> Self {
        Self { repo, audit }
    }

    pub async fn list(&self) -> Result<Vec<Setting>, AppError> {
        self.repo.list().await
    }

    pub async fn update(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        key: &str,
        value: &str,
    ) -> Result<Setting, AppError> {
        match actor_role {
            Role::SuperAdmin => {} // can update any key
            Role::Accountant => {
                if !ACCOUNTANT_ALLOWED_KEYS.contains(&key) {
                    return Err(AppError::Forbidden(format!(
                        "Accountants can only update salary-related settings"
                    )));
                }
            }
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can update settings".into())),
        }

        if value.trim().is_empty() {
            return Err(AppError::BadRequest("Value cannot be empty".into()));
        }

        // Validate numeric settings
        const NUMERIC_KEYS: &[&str] = &[
            "commission_rate",
            "trip_cap_aed",
            "salary_target_high_aed",
            "salary_fixed_car_high_aed",
            "salary_target_low_aed",
            "salary_fixed_car_low_aed",
            "car_charging_aed",
            "salik_refund_aed",
            "cash_shortfall_threshold_aed",
        ];
        if NUMERIC_KEYS.contains(&key) {
            match value.trim().parse::<f64>() {
                Ok(n) if n >= 0.0 => {}
                Ok(_) => return Err(AppError::BadRequest(format!("{} must be a non-negative number", key))),
                Err(_) => return Err(AppError::BadRequest(format!("{} must be a valid number", key))),
            }
        }

        // Capture old value before updating
        let old_value = self.repo.get(key).await?.map(|s| s.value);

        let setting = self.repo.upsert(key, value, actor_id).await?;

        self.audit.log(
            actor_id,
            actor_role,
            "setting",
            None,
            "setting.updated",
            Some(serde_json::json!({
                "key": key,
                "old_value": old_value,
                "new_value": value,
            })),
        ).await?;

        Ok(setting)
    }
}
