use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::hr::domain::entity::{LeaveRequest, LeaveStatus, LeaveType};

// ── Requests ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListLeaveQuery {
    pub driver_id: Option<Uuid>,
    pub status: Option<LeaveStatus>,
    #[serde(rename = "type")]
    pub leave_type: Option<LeaveType>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitLeaveBody {
    pub driver_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub leave_type: LeaveType,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RejectLeaveBody {
    pub rejection_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkApproveBody {
    pub request_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct BulkApproveResponse {
    pub approved_count: u64,
}

// ── Response ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LeaveResponse {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    #[serde(rename = "type")]
    pub leave_type: String,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub reason: String,
    pub status: String,
    pub actioned_by: Option<Uuid>,
    pub actioned_by_name: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: String,
}

impl From<LeaveRequest> for LeaveResponse {
    fn from(r: LeaveRequest) -> Self {
        Self {
            id: r.id,
            driver_id: r.driver_id,
            driver_name: r.driver_name,
            leave_type: format!("{:?}", r.r#type).to_lowercase(),
            from_date: r.from_date,
            to_date: r.to_date,
            reason: r.reason,
            status: format!("{:?}", r.status).to_lowercase(),
            actioned_by: r.actioned_by,
            actioned_by_name: r.actioned_by_name,
            rejection_reason: r.rejection_reason,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}
