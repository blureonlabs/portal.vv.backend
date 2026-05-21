use printpdf::*;
use rust_decimal::Decimal;
use std::io::BufWriter;

use crate::common::error::AppError;
use crate::config::AppConfig;
use crate::salary::domain::entity::Salary;

pub struct SalaryPdfService {
    http: reqwest::Client,
    supabase_url: String,
    service_role_key: String,
    bucket: String,
}

impl SalaryPdfService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            supabase_url: config.supabase_url.clone(),
            service_role_key: config.supabase_service_role_key.clone(),
            bucket: config.supabase_storage_bucket.clone(),
        }
    }

    pub async fn generate_and_upload(&self, salary: &Salary) -> Result<String, AppError> {
        let bytes = self.build_pdf(salary)?;
        let path = format!(
            "salary-slips/{}/{}.pdf",
            salary.driver_id,
            salary.period_month.format("%Y-%m")
        );
        self.upload(&path, bytes).await?;

        let public_url = format!(
            "{}/storage/v1/object/public/{}/{}",
            self.supabase_url, self.bucket, path
        );
        Ok(public_url)
    }

    fn build_pdf(&self, salary: &Salary) -> Result<Vec<u8>, AppError> {
        let title = format!("Salary Slip - {} - {}", salary.driver_name, salary.period_month.format("%B %Y"));
        let (doc, page1, layer1) = PdfDocument::new(&title, Mm(210.0), Mm(297.0), "Layer 1");
        let layer = doc.get_page(page1).get_layer(layer1);

        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| AppError::Internal(format!("Font error: {}", e)))?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| AppError::Internal(format!("Font error: {}", e)))?;

        // -- Header --
        layer.use_text("Voiture Voyages", 18.0, Mm(20.0), Mm(272.0), &font_bold);
        layer.use_text("SALARY SLIP", 22.0, Mm(140.0), Mm(272.0), &font_bold);

        // -- Divider --
        let line_pts = vec![
            (Point::new(Mm(20.0), Mm(266.0)), false),
            (Point::new(Mm(190.0), Mm(266.0)), false),
        ];
        layer.add_line(Line { points: line_pts, is_closed: false });

        // -- Driver info --
        let mut y = 258.0_f32;
        layer.use_text("Driver:", 10.0, Mm(20.0), Mm(y), &font_bold);
        layer.use_text(&salary.driver_name, 10.0, Mm(55.0), Mm(y), &font);
        y -= 6.0;

        layer.use_text("Driver ID:", 10.0, Mm(20.0), Mm(y), &font_bold);
        layer.use_text(&salary.driver_id.to_string()[..8], 10.0, Mm(55.0), Mm(y), &font);
        y -= 6.0;

        layer.use_text("Period:", 10.0, Mm(20.0), Mm(y), &font_bold);
        layer.use_text(
            &salary.period_month.format("%B %Y").to_string(),
            10.0, Mm(55.0), Mm(y), &font,
        );
        y -= 6.0;

        layer.use_text("Salary Type:", 10.0, Mm(20.0), Mm(y), &font_bold);
        layer.use_text(
            &format!("{:?}", salary.salary_type_snapshot),
            10.0, Mm(55.0), Mm(y), &font,
        );
        y -= 12.0;

        // -- Earnings section --
        layer.use_text("EARNINGS", 11.0, Mm(20.0), Mm(y), &font_bold);
        y -= 2.0;
        let line_pts = vec![
            (Point::new(Mm(20.0), Mm(y)), false),
            (Point::new(Mm(190.0), Mm(y)), false),
        ];
        layer.add_line(Line { points: line_pts, is_closed: false });
        y -= 8.0;

        let earn_row = |layer: &PdfLayerReference, label: &str, amount: Decimal, y: &mut f32, font: &IndirectFontRef| {
            layer.use_text(label, 10.0, Mm(20.0), Mm(*y), font);
            layer.use_text(&format!("{:.2}", amount), 10.0, Mm(145.0), Mm(*y), font);
            *y -= 7.0;
        };

        earn_row(&layer, "Total Earnings", salary.total_earnings_aed, &mut y, &font);
        if let Some(commission) = salary.commission_aed {
            earn_row(&layer, "Commission", commission, &mut y, &font);
        }
        if let Some(target) = salary.target_amount_aed {
            earn_row(&layer, "Target Amount", target, &mut y, &font);
        }
        if salary.incentives_aed > Decimal::ZERO {
            earn_row(&layer, "Incentives", salary.incentives_aed, &mut y, &font);
        }
        earn_row(&layer, "Base Amount", salary.base_amount_aed, &mut y, &font);
        y -= 6.0;

        // -- Deductions section --
        layer.use_text("DEDUCTIONS", 11.0, Mm(20.0), Mm(y), &font_bold);
        y -= 2.0;
        let line_pts = vec![
            (Point::new(Mm(20.0), Mm(y)), false),
            (Point::new(Mm(190.0), Mm(y)), false),
        ];
        layer.add_line(Line { points: line_pts, is_closed: false });
        y -= 8.0;

        let ded_row = |layer: &PdfLayerReference, label: &str, amount: Decimal, y: &mut f32, font: &IndirectFontRef| {
            if amount != Decimal::ZERO {
                layer.use_text(label, 10.0, Mm(20.0), Mm(*y), font);
                layer.use_text(&format!("{:.2}", amount), 10.0, Mm(145.0), Mm(*y), font);
                *y -= 7.0;
            }
        };

        ded_row(&layer, "Salik (net)", salary.salik_aed, &mut y, &font);
        ded_row(&layer, "RTA Fine", salary.rta_fine_aed, &mut y, &font);
        ded_row(&layer, "Card Service Charges", salary.card_service_charges_aed, &mut y, &font);
        if let Some(rent) = salary.room_rent_aed {
            ded_row(&layer, "Room Rent", rent, &mut y, &font);
        }
        if let Some(diff) = salary.car_charging_diff_aed {
            ded_row(&layer, "Car Charging Diff", diff, &mut y, &font);
        }
        ded_row(&layer, "Cash Not Handover", salary.cash_not_handover_aed, &mut y, &font);
        ded_row(&layer, "Advance Deduction", salary.advance_deduction_aed, &mut y, &font);
        ded_row(&layer, "Carry Forward Balance", salary.carry_forward_balance_aed, &mut y, &font);
        y -= 6.0;

        // -- Summary --
        let line_pts = vec![
            (Point::new(Mm(100.0), Mm(y)), false),
            (Point::new(Mm(190.0), Mm(y)), false),
        ];
        layer.add_line(Line { points: line_pts, is_closed: false });
        y -= 8.0;

        layer.use_text("Final Salary", 10.0, Mm(100.0), Mm(y), &font);
        layer.use_text(&format!("{:.2}", salary.final_salary_aed), 10.0, Mm(145.0), Mm(y), &font);
        y -= 8.0;

        layer.use_text("NET PAYABLE (AED)", 12.0, Mm(100.0), Mm(y), &font_bold);
        layer.use_text(&format!("{:.2}", salary.net_payable_aed), 12.0, Mm(145.0), Mm(y), &font_bold);

        // -- Footer --
        layer.use_text(
            &format!("Generated: {}", salary.generated_at.format("%d/%m/%Y %H:%M")),
            8.0, Mm(20.0), Mm(18.0), &font,
        );
        layer.use_text(
            &format!("Status: {:?}", salary.status),
            8.0, Mm(20.0), Mm(13.0), &font,
        );

        // -- Serialize --
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
