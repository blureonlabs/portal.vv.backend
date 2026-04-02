use std::sync::Arc;

use uuid::Uuid;

use crate::common::{error::AppError, types::Role};
use crate::settings::domain::{
    entity::{Setting, ACCOUNTANT_ALLOWED_KEYS},
    repository::SettingsRepository,
};

pub struct SettingsService {
    repo: Arc<dyn SettingsRepository>,
}

impl SettingsService {
    pub fn new(repo: Arc<dyn SettingsRepository>) -> Self {
        Self { repo }
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

        self.repo.upsert(key, value, actor_id).await
    }
}
