use chrono::NaiveDate;
use printpdf::*;
use rust_decimal::Decimal;
use std::io::BufWriter;

use crate::common::error::AppError;
use crate::config::AppConfig;
use crate::invoice::domain::entity::LineItem;

pub struct PdfService {
    http: reqwest::Client,
    supabase_url: String,
    service_role_key: String,
    bucket: String,
}

impl PdfService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            supabase_url: config.supabase_url.clone(),
            service_role_key: config.supabase_service_role_key.clone(),
            bucket: config.supabase_storage_bucket.clone(),
        }
    }

    /// Generate a PDF for the invoice and upload it to Supabase Storage.
    /// Returns the public URL of the uploaded PDF.
    pub async fn generate_and_upload(
        &self,
        invoice_no: &str,
        driver_name: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        line_items: &[LineItem],
        total_aed: Decimal,
        company_name: &str,
        company_address: &str,
    ) -> Result<String, AppError> {
        let bytes = self.build_pdf(
            invoice_no,
            driver_name,
            period_start,
            period_end,
            line_items,
            total_aed,
            company_name,
            company_address,
        )?;

        let path = format!("invoices/{}.pdf", invoice_no);
        self.upload(&path, bytes).await?;

        let public_url = format!(
            "{}/storage/v1/object/public/{}/{}",
            self.supabase_url, self.bucket, path
        );
        Ok(public_url)
    }

    fn build_pdf(
        &self,
        invoice_no: &str,
        driver_name: &str,
        period_start: NaiveDate,
        period_end: NaiveDate,
        line_items: &[LineItem],
        total_aed: Decimal,
        company_name: &str,
        company_address: &str,
    ) -> Result<Vec<u8>, AppError> {
        let (doc, page1, layer1) =
            PdfDocument::new(invoice_no, Mm(210.0), Mm(297.0), "Layer 1");
        let layer = doc.get_page(page1).get_layer(layer1);

        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| AppError::Internal(format!("Font error: {}", e)))?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| AppError::Internal(format!("Font error: {}", e)))?;

        // ── Header ──────────────────────────────────────────────────────────
        layer.use_text(company_name, 18.0, Mm(20.0), Mm(272.0), &font_bold);
        layer.use_text(company_address, 10.0, Mm(20.0), Mm(266.0), &font);

        layer.use_text("INVOICE", 22.0, Mm(140.0), Mm(272.0), &font_bold);
        layer.use_text(invoice_no, 11.0, Mm(140.0), Mm(265.0), &font);

        // ── Divider line ────────────────────────────────────────────────────
        let line_pts = vec![
            (Point::new(Mm(20.0), Mm(260.0)), false),
            (Point::new(Mm(190.0), Mm(260.0)), false),
        ];
        let line = Line {
            points: line_pts,
            is_closed: false,
        };
        layer.add_line(line);

        // ── Driver + Period ─────────────────────────────────────────────────
        layer.use_text("Driver:", 10.0, Mm(20.0), Mm(253.0), &font_bold);
        layer.use_text(driver_name, 10.0, Mm(50.0), Mm(253.0), &font);

        layer.use_text("Period:", 10.0, Mm(20.0), Mm(247.0), &font_bold);
        layer.use_text(
            &format!(
                "{} to {}",
                period_start.format("%d/%m/%Y"),
                period_end.format("%d/%m/%Y")
            ),
            10.0,
            Mm(50.0),
            Mm(247.0),
            &font,
        );

        // ── Line items table header ─────────────────────────────────────────
        let mut y = 233.0_f32;
        layer.use_text("Description", 10.0, Mm(20.0), Mm(y), &font_bold);
        layer.use_text("Amount (AED)", 10.0, Mm(145.0), Mm(y), &font_bold);

        let header_line_pts = vec![
            (Point::new(Mm(20.0), Mm(y - 3.0)), false),
            (Point::new(Mm(190.0), Mm(y - 3.0)), false),
        ];
        layer.add_line(Line {
            points: header_line_pts,
            is_closed: false,
        });

        y -= 10.0;

        // ── Line items ──────────────────────────────────────────────────────
        for item in line_items {
            layer.use_text(&item.description, 10.0, Mm(20.0), Mm(y), &font);
            layer.use_text(
                &format!("{:.2}", item.amount_aed),
                10.0,
                Mm(145.0),
                Mm(y),
                &font,
            );
            y -= 8.0;

            if y < 40.0_f32 {
                break; // Prevent overflow (multi-page not needed at this stage)
            }
        }

        // ── Total ───────────────────────────────────────────────────────────
        y -= 4.0;
        let total_line_pts = vec![
            (Point::new(Mm(120.0), Mm(y)), false),
            (Point::new(Mm(190.0), Mm(y)), false),
        ];
        layer.add_line(Line {
            points: total_line_pts,
            is_closed: false,
        });
        y -= 8.0;

        layer.use_text("TOTAL (AED)", 11.0, Mm(120.0), Mm(y), &font_bold);
        layer.use_text(&format!("{:.2}", total_aed), 11.0, Mm(145.0), Mm(y), &font_bold);

        // ── Footer ──────────────────────────────────────────────────────────
        layer.use_text(
            &format!("Generated: {}", chrono::Local::now().format("%d/%m/%Y")),
            8.0,
            Mm(20.0),
            Mm(15.0),
            &font,
        );

        // ── Serialize ───────────────────────────────────────────────────────
        let mut buf: Vec<u8> = Vec::new();
        doc.save(&mut BufWriter::new(std::io::Cursor::new(&mut buf)))
            .map_err(|e| AppError::Internal(format!("PDF save error: {}", e)))?;

        Ok(buf)
    }

    async fn upload(&self, path: &str, bytes: Vec<u8>) -> Result<(), AppError> {
        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.supabase_url, self.bucket, path
        );

        let res = self
            .http
            .post(&url)
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .header("Content-Type", "application/pdf")
            .header("x-upsert", "true")
            .body(bytes)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Storage upload failed: {}", e)))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Storage upload error: {}", body)));
        }

        Ok(())
    }
}
