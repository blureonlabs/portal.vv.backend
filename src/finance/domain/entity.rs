use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "expense_category", rename_all = "snake_case")]
pub enum ExpenseCategory {
    Fuel,
    Maintenance,
    Toll,
    Insurance,
    Fines,
    Other,
}

#[derive(Debug, Clone, Serialize)]
pub struct Expense {
    pub id: Uuid,
    pub driver_id: Option<Uuid>,
    pub driver_name: Option<String>,
    pub entered_by: Uuid,
    pub amount_aed: Decimal,
    pub category: ExpenseCategory,
    pub date: NaiveDate,
    pub receipt_url: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CashHandover {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub amount_aed: Decimal,
    pub submitted_at: DateTime<Utc>,
    pub verified_by: Uuid,
    pub verifier_name: String,
}

#[derive(Debug, Clone)]
pub struct CreateExpense {
    pub driver_id: Option<Uuid>,
    pub entered_by: Uuid,
    pub amount_aed: Decimal,
    pub category: ExpenseCategory,
    pub date: NaiveDate,
    pub receipt_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateHandover {
    pub driver_id: Uuid,
    pub amount_aed: Decimal,
    pub verified_by: Uuid,
}
