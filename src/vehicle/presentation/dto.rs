use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::deserialize::empty_string_as_none_date;
use crate::vehicle::domain::entity::{Vehicle, VehicleAssignment, VehicleServiceRecord, VehicleStatus};

// ── Requests ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateVehicleRequest {
    pub plate_number: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none_date")]
    pub registration_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "empty_string_as_none_date")]
    pub registration_expiry: Option<NaiveDate>,
    #[serde(default, deserialize_with = "empty_string_as_none_date")]
    pub insurance_expiry: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVehicleRequest {
    pub plate_number: String,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none_date")]
    pub registration_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "empty_string_as_none_date")]
    pub registration_expiry: Option<NaiveDate>,
    #[serde(default, deserialize_with = "empty_string_as_none_date")]
    pub insurance_expiry: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct AssignDriverRequest {
    pub driver_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AddServiceRecordRequest {
    pub service_date: NaiveDate,
    pub service_type: String,
    pub description: Option<String>,
    pub cost: Decimal,
    pub next_due: Option<NaiveDate>,
}

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct VehicleResponse {
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
    pub assigned_driver_id: Option<Uuid>,
    pub assigned_driver_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Vehicle> for VehicleResponse {
    fn from(v: Vehicle) -> Self {
        Self {
            id: v.id,
            plate_number: v.plate_number,
            make: v.make,
            model: v.model,
            year: v.year,
            color: v.color,
            registration_date: v.registration_date,
            registration_expiry: v.registration_expiry,
            insurance_expiry: v.insurance_expiry,
            status: v.status,
            is_active: v.is_active,
            assigned_driver_id: v.assigned_driver_id,
            assigned_driver_name: v.assigned_driver_name,
            created_at: v.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VehicleAssignmentResponse {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub assigned_at: DateTime<Utc>,
    pub unassigned_at: Option<DateTime<Utc>>,
    pub assigned_by: Uuid,
}

impl From<VehicleAssignment> for VehicleAssignmentResponse {
    fn from(a: VehicleAssignment) -> Self {
        Self {
            id: a.id,
            vehicle_id: a.vehicle_id,
            driver_id: a.driver_id,
            driver_name: a.driver_name,
            assigned_at: a.assigned_at,
            unassigned_at: a.unassigned_at,
            assigned_by: a.assigned_by,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VehicleServiceResponse {
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

impl From<VehicleServiceRecord> for VehicleServiceResponse {
    fn from(r: VehicleServiceRecord) -> Self {
        Self {
            id: r.id,
            vehicle_id: r.vehicle_id,
            service_date: r.service_date,
            service_type: r.service_type,
            description: r.description,
            cost: r.cost,
            next_due: r.next_due,
            logged_by: r.logged_by,
            created_at: r.created_at,
        }
    }
}
