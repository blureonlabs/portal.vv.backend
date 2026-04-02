use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub description: String,
    pub amount_aed: Decimal,
}

#[derive(Debug, Clone)]
pub struct Invoice {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub invoice_no: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub line_items: Vec<LineItem>,
    pub total_aed: Decimal,
    pub pdf_url: Option<String>,
    pub generated_by: Uuid,
    pub generated_by_name: String,
    pub created_at: DateTime<Utc>,
}

pub struct CreateInvoice {
    pub driver_id: Uuid,
    pub invoice_no: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub line_items: Vec<LineItem>,
    pub total_aed: Decimal,
    pub pdf_url: Option<String>,
    pub generated_by: Uuid,
}
