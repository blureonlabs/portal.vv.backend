use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::invoice::domain::{
    entity::{CreateInvoice, Invoice, LineItem},
    repository::InvoiceRepository,
};

pub struct PgInvoiceRepository {
    pool: PgPool,
}

impl PgInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Row type for sqlx — JSONB comes back as serde_json::Value
struct InvoiceRow {
    id: Uuid,
    driver_id: Uuid,
    driver_name: String,
    invoice_no: String,
    period_start: NaiveDate,
    period_end: NaiveDate,
    line_items_json: serde_json::Value,
    total_aed: rust_decimal::Decimal,
    pdf_url: Option<String>,
    generated_by: Uuid,
    generated_by_name: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

fn row_to_invoice(r: InvoiceRow) -> Result<Invoice, AppError> {
    let line_items: Vec<LineItem> = serde_json::from_value(r.line_items_json)
        .map_err(|e| AppError::Internal(format!("Invalid line_items JSON: {}", e)))?;
    Ok(Invoice {
        id: r.id,
        driver_id: r.driver_id,
        driver_name: r.driver_name,
        invoice_no: r.invoice_no,
        period_start: r.period_start,
        period_end: r.period_end,
        line_items,
        total_aed: r.total_aed,
        pdf_url: r.pdf_url,
        generated_by: r.generated_by,
        generated_by_name: r.generated_by_name,
        created_at: r.created_at,
    })
}

#[async_trait]
impl InvoiceRepository for PgInvoiceRepository {
    async fn list(&self, driver_id: Option<Uuid>) -> Result<Vec<Invoice>, AppError> {
        let rows = sqlx::query_as!(
            InvoiceRow,
            r#"
            SELECT
                i.id, i.driver_id,
                pd.full_name AS driver_name,
                i.invoice_no,
                i.period_start, i.period_end,
                i.line_items_json,
                i.total_aed,
                i.pdf_url,
                i.generated_by,
                pg.full_name AS generated_by_name,
                i.created_at
            FROM invoices i
            JOIN drivers d ON d.id = i.driver_id
            JOIN profiles pd ON pd.id = d.profile_id
            JOIN profiles pg ON pg.id = i.generated_by
            WHERE ($1::uuid IS NULL OR i.driver_id = $1)
            ORDER BY i.created_at DESC
            "#,
            driver_id as Option<Uuid>
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_invoice).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Invoice, AppError> {
        let row = sqlx::query_as!(
            InvoiceRow,
            r#"
            SELECT
                i.id, i.driver_id,
                pd.full_name AS driver_name,
                i.invoice_no,
                i.period_start, i.period_end,
                i.line_items_json,
                i.total_aed,
                i.pdf_url,
                i.generated_by,
                pg.full_name AS generated_by_name,
                i.created_at
            FROM invoices i
            JOIN drivers d ON d.id = i.driver_id
            JOIN profiles pd ON pd.id = d.profile_id
            JOIN profiles pg ON pg.id = i.generated_by
            WHERE i.id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;

        row_to_invoice(row)
    }

    async fn create(&self, payload: CreateInvoice) -> Result<Invoice, AppError> {
        let line_items_json = serde_json::to_value(&payload.line_items)
            .map_err(|e| AppError::Internal(format!("Failed to serialize line items: {}", e)))?;

        let row = sqlx::query_as!(
            InvoiceRow,
            r#"
            WITH ins AS (
                INSERT INTO invoices
                    (driver_id, invoice_no, period_start, period_end, line_items_json, total_aed, pdf_url, generated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
            )
            SELECT
                ins.id, ins.driver_id,
                pd.full_name AS driver_name,
                ins.invoice_no,
                ins.period_start, ins.period_end,
                ins.line_items_json,
                ins.total_aed,
                ins.pdf_url,
                ins.generated_by,
                pg.full_name AS generated_by_name,
                ins.created_at
            FROM ins
            JOIN drivers d ON d.id = ins.driver_id
            JOIN profiles pd ON pd.id = d.profile_id
            JOIN profiles pg ON pg.id = ins.generated_by
            "#,
            payload.driver_id,
            payload.invoice_no,
            payload.period_start,
            payload.period_end,
            line_items_json,
            payload.total_aed,
            payload.pdf_url,
            payload.generated_by
        )
        .fetch_one(&self.pool)
        .await?;

        row_to_invoice(row)
    }

    async fn next_sequence(&self, period_start: NaiveDate) -> Result<u32, AppError> {
        let month = period_start.format("%Y-%m").to_string();
        let seq: i32 = sqlx::query_scalar(
            "INSERT INTO invoice_counters (month, last_seq) VALUES ($1, 1)
             ON CONFLICT (month) DO UPDATE SET last_seq = invoice_counters.last_seq + 1
             RETURNING last_seq"
        )
        .bind(month)
        .fetch_one(&self.pool)
        .await?;

        Ok(seq as u32)
    }
}
