use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::finance::domain::entity::{CashHandover, Expense, ExpenseCategory};

// ── Requests ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub driver_id: Option<Uuid>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CreateExpenseRequest {
    pub driver_id: Option<Uuid>,
    pub amount_aed: Decimal,
    pub category: ExpenseCategory,
    pub date: NaiveDate,
    pub receipt_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHandoverRequest {
    pub driver_id: Uuid,
    pub amount_aed: Decimal,
}

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ExpenseResponse {
    pub id: Uuid,
    pub driver_id: Option<Uuid>,
    pub driver_name: Option<String>,
    pub amount_aed: Decimal,
    pub category: String,
    pub date: NaiveDate,
    pub receipt_url: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<Expense> for ExpenseResponse {
    fn from(e: Expense) -> Self {
        Self {
            id: e.id,
            driver_id: e.driver_id,
            driver_name: e.driver_name,
            amount_aed: e.amount_aed,
            category: format!("{:?}", e.category).to_lowercase(),
            date: e.date,
            receipt_url: e.receipt_url,
            notes: e.notes,
            created_at: e.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HandoverResponse {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub amount_aed: Decimal,
    pub submitted_at: String,
    pub verified_by: Uuid,
    pub verifier_name: String,
}

impl From<CashHandover> for HandoverResponse {
    fn from(h: CashHandover) -> Self {
        Self {
            id: h.id,
            driver_id: h.driver_id,
            driver_name: h.driver_name,
            amount_aed: h.amount_aed,
            submitted_at: h.submitted_at.to_rfc3339(),
            verified_by: h.verified_by,
            verifier_name: h.verifier_name,
        }
    }
}
