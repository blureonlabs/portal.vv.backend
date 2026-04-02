use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "vehicle_status", rename_all = "snake_case")]
pub enum VehicleStatus {
    Available,
    Assigned,
    Inactive,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Vehicle {
    pub id: Uuid,
    pub plate_number: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: Option<String>,
    pub registration_date: Option<NaiveDate>,
    pub registration_expiry: Option<NaiveDate>,
    pub insurance_expiry: Option<NaiveDate>,
    pub status: VehicleStatus,
    pub is_active: bool,
    // Current assignment (null if not assigned)
    pub assigned_driver_id: Option<Uuid>,
    pub assigned_driver_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VehicleAssignment {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub assigned_at: DateTime<Utc>,
    pub unassigned_at: Option<DateTime<Utc>>,
    pub assigned_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VehicleServiceRecord {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub service_date: NaiveDate,
    pub service_type: String,
    pub description: Option<String>,
    pub cost: Decimal,
    pub next_due: Option<NaiveDate>,
    pub logged_by: Uuid,
    pub created_at: DateTime<Utc>,
}
