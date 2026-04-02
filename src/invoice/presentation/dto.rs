use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::invoice::domain::entity::Invoice;

// ── Requests ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListInvoicesQuery {
    pub driver_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateInvoiceBody {
    pub driver_id: Uuid,
    pub driver_name: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub line_items: Vec<LineItemDto>,
}

#[derive(Debug, Deserialize)]
pub struct LineItemDto {
    pub description: String,
    pub amount_aed: Decimal,
}

// ── Responses ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub invoice_no: String,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub line_items: Vec<LineItemResponse>,
    pub total_aed: Decimal,
    pub pdf_url: Option<String>,
    pub generated_by: Uuid,
    pub generated_by_name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct LineItemResponse {
    pub description: String,
    pub amount_aed: Decimal,
}

impl From<Invoice> for InvoiceResponse {
    fn from(inv: Invoice) -> Self {
        Self {
            id: inv.id,
            driver_id: inv.driver_id,
            driver_name: inv.driver_name,
            invoice_no: inv.invoice_no,
            period_start: inv.period_start,
            period_end: inv.period_end,
            line_items: inv.line_items.into_iter().map(|li| LineItemResponse {
                description: li.description,
                amount_aed: li.amount_aed,
            }).collect(),
            total_aed: inv.total_aed,
            pdf_url: inv.pdf_url,
            generated_by: inv.generated_by,
            generated_by_name: inv.generated_by_name,
            created_at: inv.created_at.to_rfc3339(),
        }
    }
}
