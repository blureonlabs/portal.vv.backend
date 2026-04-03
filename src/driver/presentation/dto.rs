use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::types::SalaryType;
use crate::driver::domain::entity::{Driver, DriverEdit};

#[derive(Debug, Deserialize)]
pub struct CreateDriverRequest {
    pub profile_id: Uuid,
    pub nationality: String,
    pub salary_type: SalaryType,
}

/// Create driver with a new auth account in one step (super_admin use)
#[derive(Debug, Deserialize)]
pub struct CreateDriverWithAccountRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    #[serde(default)]
    pub phone: Option<String>,
    pub nationality: String,
    pub salary_type: SalaryType,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDriverRequest {
    pub nationality: String,
    pub salary_type: SalaryType,
}

#[derive(Debug, Serialize)]
pub struct DriverResponse {
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

impl From<Driver> for DriverResponse {
    fn from(d: Driver) -> Self {
        Self {
            id: d.id,
            profile_id: d.profile_id,
            full_name: d.full_name,
            email: d.email,
            nationality: d.nationality,
            salary_type: d.salary_type,
            is_active: d.is_active,
            self_entry_enabled: d.self_entry_enabled,
            created_at: d.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetSelfEntryRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct DriverEditResponse {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub changed_by: Uuid,
    pub field: String,
    pub old_val: Option<String>,
    pub new_val: Option<String>,
    pub changed_at: DateTime<Utc>,
}

impl From<DriverEdit> for DriverEditResponse {
    fn from(e: DriverEdit) -> Self {
        Self {
            id: e.id,
            driver_id: e.driver_id,
            changed_by: e.changed_by,
            field: e.field,
            old_val: e.old_val,
            new_val: e.new_val,
            changed_at: e.changed_at,
        }
    }
}
