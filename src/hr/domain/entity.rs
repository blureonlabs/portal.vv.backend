use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "leave_type", rename_all = "snake_case")]
pub enum LeaveType {
    Leave,
    Permission,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "leave_status", rename_all = "snake_case")]
pub enum LeaveStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct LeaveRequest {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub r#type: LeaveType,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub reason: String,
    pub status: LeaveStatus,
    pub actioned_by: Option<Uuid>,
    pub actioned_by_name: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateLeaveRequest {
    pub driver_id: Uuid,
    pub r#type: LeaveType,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub reason: String,
}

pub struct ActionLeaveRequest {
    pub id: Uuid,
    pub actioned_by: Uuid,
    pub rejection_reason: Option<String>,
}
