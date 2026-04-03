use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "advance_status", rename_all = "snake_case")]
pub enum AdvanceStatus {
    Pending,
    Approved,
    Rejected,
    Paid,
}

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    BankTransfer,
}

#[derive(Debug, Clone)]
pub struct Advance {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateAdvance {
    pub driver_id: Uuid,
    pub amount_aed: Decimal,
    pub reason: String,
}

pub struct ApproveAdvance {
    pub id: Uuid,
    pub actioned_by: Uuid,
}

pub struct RejectAdvance {
    pub id: Uuid,
    pub actioned_by: Uuid,
    pub rejection_reason: String,
}

pub struct PayAdvance {
    pub id: Uuid,
    pub actioned_by: Uuid,
    pub payment_date: NaiveDate,
    pub method: PaymentMethod,
    pub salary_period: Option<NaiveDate>,
}
