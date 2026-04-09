use std::sync::Arc;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use uuid::Uuid;

use tracing;

use crate::audit::application::service::AuditService;
use crate::common::{error::AppError, ports::DeductionPort, types::{Role, SalaryType}};
use crate::salary::domain::{
    entity::{CreateSalary, Salary},
    repository::SalaryRepository,
};
use crate::settings::domain::repository::SettingsRepository;

pub struct SalaryService {
    repo: Arc<dyn SalaryRepository>,
    settings: Arc<dyn SettingsRepository>,
    deduction_port: Arc<dyn DeductionPort>,
    audit: Arc<AuditService>,
}

impl SalaryService {
    pub fn new(
        repo: Arc<dyn SalaryRepository>,
        settings: Arc<dyn SettingsRepository>,
        deduction_port: Arc<dyn DeductionPort>,
        audit: Arc<AuditService>,
    ) -> Self {
        Self { repo, settings, deduction_port, audit }
    }

    pub async fn list(
        &self,
        driver_id: Option<Uuid>,
        month: Option<NaiveDate>,
    ) -> Result<Vec<Salary>, AppError> {
        self.repo.list(driver_id, month).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Salary, AppError> {
        self.repo.find_by_id(id).await
    }

    /// Re-computes salary on every call (idempotent upsert).
    pub async fn generate(&self, actor_role: &Role, req: GenerateRequest) -> Result<Salary, AppError> {
        // 1. Load settings
        let settings = self.settings.list().await?;
        let get_setting = |key: &str, default: f64| -> Decimal {
            settings.iter()
                .find(|s| s.key == key)
                .and_then(|s| s.value.parse::<f64>().ok())
                .and_then(Decimal::from_f64)
                .unwrap_or_else(|| Decimal::from_f64(default).unwrap())
        };

        let global_commission_rate = get_setting("commission_rate",          0.75);
        let target_high_aed        = get_setting("salary_target_high_aed",  0.0);
        let target_low_aed         = get_setting("salary_target_low_aed",   0.0);
        let fixed_car_low_aed      = get_setting("salary_fixed_car_low_aed", 0.0);

        // Per-driver overrides: driver's commission_rate takes precedence over global setting.
        let commission_rate = req.driver_commission_rate.unwrap_or(global_commission_rate);

        // Room rent: if request body provides a value > 0, use it as override; otherwise use the driver's stored value.
        let effective_room_rent = match req.room_rent_aed {
            Some(v) if v > Decimal::ZERO => v,
            _ => req.driver_room_rent_aed,
        };

        // 2. Advance deductions
        let advance_deduction_aed = self.deduction_port
            .get_advance_deductions(req.driver_id, req.period_month)
            .await?;

        let advance_ids = self.deduction_port
            .get_advance_ids_for_period(req.driver_id, req.period_month)
            .await?;

        let deductions_json = if advance_ids.is_empty() {
            None
        } else {
            Some(serde_json::json!({ "advance_ids": advance_ids }))
        };

        // 3. Derived totals from inputs
        let salik_aed = req.salik_used_aed - req.salik_refund_aed;

        let car_charging_diff_aed = req.car_charging_used_aed.map(|used| req.car_charging_aed - used);
        let cash_diff_aed         = req.total_cash_submit_aed.map(|sub| req.total_cash_received_aed - sub);

        // 4. Formula
        let (base_amount_aed, commission_aed, target_amount_aed, fixed_car_charging_aed, final_salary_aed) =
            match req.salary_type {
                SalaryType::Commission => {
                    let commission = req.total_earnings_aed * commission_rate;
                    let final_sal = commission
                        - salik_aed
                        - req.rta_fine_aed
                        - req.card_service_charges_aed
                        - effective_room_rent;
                    (commission, Some(commission), None, None, final_sal)
                }
                SalaryType::TargetHigh => {
                    let base = target_high_aed
                        + car_charging_diff_aed.unwrap_or(Decimal::ZERO);
                    let final_sal = base
                        - salik_aed
                        - req.rta_fine_aed
                        - req.card_service_charges_aed
                        - effective_room_rent
                        - req.cash_not_handover_aed;
                    (base, None, Some(target_high_aed), None, final_sal)
                }
                SalaryType::TargetLow => {
                    let base = target_low_aed
                        + car_charging_diff_aed.unwrap_or(Decimal::ZERO);
                    let final_sal = base
                        - salik_aed
                        - req.rta_fine_aed
                        - req.card_service_charges_aed
                        - effective_room_rent
                        - req.cash_not_handover_aed;
                    (base, None, Some(target_low_aed), Some(fixed_car_low_aed), final_sal)
                }
            };

        let net_payable_aed = final_salary_aed - advance_deduction_aed;

        let payload = CreateSalary {
            driver_id:               req.driver_id,
            period_month:            req.period_month,
            salary_type_snapshot:    req.salary_type,
            total_earnings_aed:      req.total_earnings_aed,
            total_cash_received_aed: req.total_cash_received_aed,
            total_cash_submit_aed:   req.total_cash_submit_aed,
            cash_not_handover_aed:   req.cash_not_handover_aed,
            cash_diff_aed,
            car_charging_aed:        req.car_charging_aed,
            car_charging_used_aed:   req.car_charging_used_aed,
            car_charging_diff_aed,
            salik_used_aed:          req.salik_used_aed,
            salik_refund_aed:        req.salik_refund_aed,
            salik_aed,
            rta_fine_aed:            req.rta_fine_aed,
            card_service_charges_aed: req.card_service_charges_aed,
            room_rent_aed:           if effective_room_rent > Decimal::ZERO { Some(effective_room_rent) } else { None },
            target_amount_aed,
            fixed_car_charging_aed,
            commission_aed,
            base_amount_aed,
            final_salary_aed,
            advance_deduction_aed,
            net_payable_aed,
            deductions_json,
            generated_by:            req.generated_by,
        };

        let mut salary = self.repo.upsert(payload).await?;

        // Set a placeholder slip_url (real PDF generation to follow in a future sprint).
        // Path convention: salary-slips/{driver_id}/{period_month}.pdf
        let slip_path = format!(
            "salary-slips/{}/{}.pdf",
            salary.driver_id,
            salary.period_month.format("%Y-%m")
        );
        // slip_url is stored as the storage path; the frontend constructs the full URL.
        if let Err(e) = self.repo.update_slip_url(salary.id, &slip_path).await {
            tracing::warn!("Failed to set slip_url for salary {}: {}", salary.id, e);
        } else {
            salary.slip_url = Some(slip_path);
        }

        self.audit.log(req.generated_by, actor_role, "salary", Some(salary.id), "salary.generated",
            Some(serde_json::json!({
                "driver_id": req.driver_id,
                "period_month": req.period_month,
                "net_payable_aed": net_payable_aed
            }))).await?;

        Ok(salary)
    }
}

pub struct GenerateRequest {
    pub driver_id:                Uuid,
    pub period_month:             NaiveDate,
    pub salary_type:              SalaryType,
    pub total_earnings_aed:       Decimal,
    pub total_cash_received_aed:  Decimal,
    pub total_cash_submit_aed:    Option<Decimal>,
    pub cash_not_handover_aed:    Decimal,
    pub car_charging_aed:         Decimal,
    pub car_charging_used_aed:    Option<Decimal>,
    pub salik_used_aed:           Decimal,
    pub salik_refund_aed:         Decimal,
    pub rta_fine_aed:             Decimal,
    pub card_service_charges_aed: Decimal,
    /// Override room rent from the request body. If None or 0, falls back to the driver's stored room_rent_aed.
    pub room_rent_aed:            Option<Decimal>,
    /// Driver's per-record room rent (fetched by handler from the driver row).
    pub driver_room_rent_aed:     Decimal,
    /// Driver's per-record commission rate (None = use global setting).
    pub driver_commission_rate:   Option<Decimal>,
    pub generated_by:             Uuid,
}
