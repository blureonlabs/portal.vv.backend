use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Setting {
    pub id: Uuid,
    pub key: String,
    pub value: String,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

/// Keys that accountants are permitted to update (salary formula constants).
pub const ACCOUNTANT_ALLOWED_KEYS: &[&str] = &[
    "salary_target_high_aed",
    "salary_fixed_car_high_aed",
    "salary_target_low_aed",
    "salary_fixed_car_low_aed",
];
