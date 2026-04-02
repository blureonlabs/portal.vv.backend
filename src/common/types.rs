use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum Role {
    SuperAdmin,
    Accountant,
    Hr,
    Driver,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "salary_type", rename_all = "snake_case")]
pub enum SalaryType {
    Commission,
    TargetHigh,
    TargetLow,
}

/// Injected into every authenticated handler via FromRequest extractor.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub role: Role,
    pub email: String,
}
