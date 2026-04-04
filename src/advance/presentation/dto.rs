use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::advance::domain::entity::{Advance, AdvanceStatus, PaymentMethod};
use crate::common::deserialize::{empty_string_as_none_date, empty_string_as_none_uuid};

// ── Requests ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListAdvancesQuery {
    pub driver_id: Option<Uuid>,
    pub status: Option<AdvanceStatus>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RequestAdvanceBody {
    #[serde(default, deserialize_with = "empty_string_as_none_uuid")]
    pub driver_id: Option<Uuid>,
    pub amount_aed: Decimal,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RejectAdvanceBody {
    pub rejection_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct PayAdvanceBody {
    pub payment_date: NaiveDate,
    pub method: PaymentMethod,
    #[serde(default, deserialize_with = "empty_string_as_none_date")]
    pub salary_period: Option<NaiveDate>,
}

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AdvanceResponse {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub amount_aed: Decimal,
    pub reason: String,
    pub status: AdvanceStatus,
    pub rejection_reason: Option<String>,
    pub payment_date: Option<NaiveDate>,
    pub method: Option<PaymentMethod>,
    pub carry_forward_aed: Decimal,
    pub salary_period: Option<NaiveDate>,
    pub actioned_by: Option<Uuid>,
    pub actioned_by_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Advance> for AdvanceResponse {
    fn from(a: Advance) -> Self {
        Self {
            id: a.id,
            driver_id: a.driver_id,
            driver_name: a.driver_name,
            amount_aed: a.amount_aed,
            reason: a.reason,
            status: a.status,
            rejection_reason: a.rejection_reason,
            payment_date: a.payment_date,
            method: a.method,
            carry_forward_aed: a.carry_forward_aed,
            salary_period: a.salary_period,
            actioned_by: a.actioned_by,
            actioned_by_name: a.actioned_by_name,
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        }
    }
}
