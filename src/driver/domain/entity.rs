use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::common::types::SalaryType;

/// Full driver record — joined with profiles for name/email
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Driver {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub nationality: String,
    pub salary_type: SalaryType,
    pub is_active: bool,
    pub self_entry_enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DriverEdit {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub changed_by: Uuid,
    pub field: String,
    pub old_val: Option<String>,
    pub new_val: Option<String>,
    pub changed_at: DateTime<Utc>,
}
