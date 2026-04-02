use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::{error::AppError, types::Role};
use crate::invoice::domain::{
    entity::{CreateInvoice, Invoice, LineItem},
    repository::InvoiceRepository,
};
use crate::invoice::infrastructure::PdfService;

pub struct InvoiceService {
    repo: Arc<dyn InvoiceRepository>,
    pdf: Arc<PdfService>,
    company_name: String,
    company_address: String,
}

impl InvoiceService {
    pub fn new(
        repo: Arc<dyn InvoiceRepository>,
        pdf: Arc<PdfService>,
        company_name: String,
        company_address: String,
    ) -> Self {
        Self { repo, pdf, company_name, company_address }
    }

    pub async fn list(
        &self,
        actor_role: &Role,
        actor_driver_id: Option<Uuid>,
        driver_id: Option<Uuid>,
    ) -> Result<Vec<Invoice>, AppError> {
        let effective = if *actor_role == Role::Driver {
            actor_driver_id
        } else {
            driver_id
        };
        self.repo.list(effective).await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Invoice, AppError> {
        self.repo.find_by_id(id).await
    }

    pub async fn generate(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        driver_id: Uuid,
        driver_name: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        line_items: Vec<LineItem>,
    ) -> Result<Invoice, AppError> {
        match actor_role {
            Role::SuperAdmin | Role::Accountant => {}
            _ => return Err(AppError::Forbidden("Only super_admin or accountant can generate invoices".into())),
        }

        if period_start > period_end {
            return Err(AppError::BadRequest("period_start must be on or before period_end".into()));
        }

        let total_aed: Decimal = line_items.iter().map(|li| li.amount_aed).sum();

        let seq = self.repo.next_sequence(period_start).await?;
        let invoice_no = format!("INV-{}-{:04}", period_start.format("%Y-%m"), seq);

        // Generate PDF and upload to Supabase Storage
        let pdf_url = self
            .pdf
            .generate_and_upload(
                &invoice_no,
                driver_name,
                period_start,
                period_end,
                &line_items,
                total_aed,
                &self.company_name,
                &self.company_address,
            )
            .await
            .ok(); // PDF upload failure is non-fatal; invoice is still created

        self.repo
            .create(CreateInvoice {
                driver_id,
                invoice_no,
                period_start,
                period_end,
                line_items,
                total_aed,
                pdf_url,
                generated_by: actor_id,
            })
            .await
    }
}
