use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use uuid::Uuid;

use tracing;

use crate::audit::application::service::AuditService;
use crate::common::{error::AppError, ports::DeductionPort, types::{Role, SalaryType}};
use crate::salary::domain::{
    entity::{CreateSalary, Salary, SalaryStatus},
    repository::SalaryRepository,
};
use crate::salary::infrastructure::pdf::SalaryPdfService;
use crate::settings::domain::repository::SettingsRepository;

pub struct SalaryService {
    repo: Arc<dyn SalaryRepository>,
    settings: Arc<dyn SettingsRepository>,
    deduction_port: Arc<dyn DeductionPort>,
    audit: Arc<AuditService>,
    pdf: Arc<SalaryPdfService>,
}

impl SalaryService {
    pub fn new(
        repo: Arc<dyn SalaryRepository>,
        settings: Arc<dyn SettingsRepository>,
        deduction_port: Arc<dyn DeductionPort>,
        audit: Arc<AuditService>,
        pdf: Arc<SalaryPdfService>,
    ) -> Self {
        Self { repo, settings, deduction_port, audit, pdf }
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

        let global_commission_rate = get_setting("commission_rate",              0.30);
        let target_high_aed        = get_setting("salary_target_high_aed",    12300.0);
        let target_low_aed         = get_setting("salary_target_low_aed",     6600.0);
        let fixed_car_low_aed      = get_setting("salary_fixed_car_low_aed",  800.0);
        let fixed_car_high_aed     = get_setting("salary_fixed_car_high_aed", 1600.0);

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

        // 2b. Carry-forward from previous month's negative balance
        let carry_forward_balance_aed = {
            let prev = prev_month(req.period_month);
            match self.repo.find_by_driver_month(req.driver_id, prev).await? {
                Some(prev_salary) if prev_salary.net_payable_aed < Decimal::ZERO => prev_salary.net_payable_aed.abs(),
                _ => Decimal::ZERO,
            }
        };

        // 3. Derived totals from inputs
        let salik_aed = req.salik_used_aed - req.salik_refund_aed;
        let cash_submit_aed = req.total_cash_received_aed - req.cash_not_handover_aed;

        let car_charging_diff_aed = req.car_charging_used_aed.map(|used| req.car_charging_aed - used);
        let cash_diff_aed         = req.total_cash_submit_aed.map(|sub| req.total_cash_received_aed - sub);

        // 4. Formula
        let (base_amount_aed, commission_aed, target_amount_aed, fixed_car_charging_aed, final_salary_aed) =
            match req.salary_type {
                SalaryType::Commission => {
                    // Step 1: Base = Total Earnings - Car Charging - Salik
                    let base = req.total_earnings_aed - req.car_charging_aed - salik_aed;
                    // Step 2: Commission = Base × Commission %
                    let commission = base * commission_rate;
                    // Step 3: Final = Commission + Incentives - RTA Fine - Cash Submit - Card Charges
                    // Room rent is NOT deducted in commission type per spec
                    let final_sal = commission
                        + req.incentives_aed
                        - req.rta_fine_aed
                        - cash_submit_aed
                        - req.card_service_charges_aed;
                    (base, Some(commission), None, None, final_sal)
                }
                SalaryType::TargetHigh => {
                    // Step 1: Adjustments
                    let car_adj = fixed_car_high_aed - req.car_charging_used_aed.unwrap_or(Decimal::ZERO);
                    let cash_adj = req.total_cash_received_aed - req.cash_not_handover_aed;
                    // Step 2: Base = (Total Earnings - Target) - Car Adj - Cash Adj
                    let base = (req.total_earnings_aed - target_high_aed) - car_adj - cash_adj;
                    // Step 3: Final = Base + Incentives - RTA Fine - Salik - Card Charges - Room Rent
                    let final_sal = base
                        + req.incentives_aed
                        - req.rta_fine_aed
                        - salik_aed
                        - req.card_service_charges_aed
                        - effective_room_rent;
                    (base, None, Some(target_high_aed), Some(fixed_car_high_aed), final_sal)
                }
                SalaryType::TargetLow => {
                    // Same logic as TargetHigh with different constants
                    let car_adj = fixed_car_low_aed - req.car_charging_used_aed.unwrap_or(Decimal::ZERO);
                    let cash_adj = req.total_cash_received_aed - req.cash_not_handover_aed;
                    let base = (req.total_earnings_aed - target_low_aed) - car_adj - cash_adj;
                    let final_sal = base
                        + req.incentives_aed
                        - req.rta_fine_aed
                        - salik_aed
                        - req.card_service_charges_aed
                        - effective_room_rent;
                    (base, None, Some(target_low_aed), Some(fixed_car_low_aed), final_sal)
                }
            };

        let net_payable_aed = final_salary_aed - advance_deduction_aed - carry_forward_balance_aed;

        let payload = CreateSalary {
            driver_id:               req.driver_id,
            period_month:            req.period_month,
            salary_type_snapshot:    req.salary_type,
            total_earnings_aed:      req.total_earnings_aed,
            total_cash_received_aed: req.total_cash_received_aed,
            total_cash_submit_aed:   Some(cash_submit_aed),
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
            carry_forward_balance_aed,
            adjusted_from_id:        None,
            deductions_json,
            generated_by:            req.generated_by,
            incentives_aed:          req.incentives_aed,
        };

        // Guard: do not overwrite an already-approved or paid salary
        if let Some(existing) = self.repo.find_by_driver_month(req.driver_id, req.period_month).await? {
            if existing.status != crate::salary::domain::entity::SalaryStatus::Draft {
                return Err(AppError::Conflict(
                    "Salary for this period is already approved or paid and cannot be regenerated".into(),
                ));
            }
        }

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

    pub async fn approve(&self, actor_id: Uuid, actor_role: &Role, id: Uuid) -> Result<Salary, AppError> {
        let salary = self.repo.approve(id, actor_id).await?;
        self.audit.log(actor_id, actor_role, "salary", Some(id), "salary.approved",
            Some(serde_json::json!({
                "salary_id": id,
                "approved_by": actor_id
            }))).await?;
        Ok(salary)
    }

    pub async fn mark_paid(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        id: Uuid,
        payment_date: NaiveDate,
        payment_mode: String,
        payment_reference: Option<String>,
        notes: Option<String>,
    ) -> Result<Salary, AppError> {
        let salary = self.repo.mark_paid(id, payment_date, payment_mode.clone(), payment_reference.clone(), notes).await?;
        self.audit.log(actor_id, actor_role, "salary", Some(id), "salary.paid",
            Some(serde_json::json!({
                "salary_id": id,
                "payment_date": payment_date,
                "payment_mode": payment_mode,
                "payment_reference": payment_reference
            }))).await?;
        Ok(salary)
    }

    pub async fn edit_salary(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        salary_id: Uuid,
        fields: EditSalaryFields,
    ) -> Result<Salary, AppError> {
        // 1. Fetch existing salary, guard draft
        let existing = self.repo.find_by_id(salary_id).await?;
        if existing.status != SalaryStatus::Draft {
            return Err(AppError::BadRequest("Only draft salaries can be edited".into()));
        }

        // 2. Load settings (same as generate)
        let settings = self.settings.list().await?;
        let get_setting = |key: &str, default: f64| -> Decimal {
            settings.iter()
                .find(|s| s.key == key)
                .and_then(|s| s.value.parse::<f64>().ok())
                .and_then(Decimal::from_f64)
                .unwrap_or_else(|| Decimal::from_f64(default).unwrap())
        };

        let global_commission_rate = get_setting("commission_rate",              0.30);
        let target_high_aed        = get_setting("salary_target_high_aed",    12300.0);
        let target_low_aed         = get_setting("salary_target_low_aed",     6600.0);
        let fixed_car_low_aed      = get_setting("salary_fixed_car_low_aed",  800.0);
        let fixed_car_high_aed     = get_setting("salary_fixed_car_high_aed", 1600.0);

        let commission_rate = fields.driver_commission_rate.unwrap_or(global_commission_rate);
        let effective_room_rent = match fields.room_rent_aed {
            Some(v) if v > Decimal::ZERO => v,
            _ => fields.driver_room_rent_aed,
        };

        // 3. Recompute advance deductions and carry-forward
        let advance_deduction_aed = self.deduction_port
            .get_advance_deductions(existing.driver_id, existing.period_month)
            .await?;
        let advance_ids = self.deduction_port
            .get_advance_ids_for_period(existing.driver_id, existing.period_month)
            .await?;
        let deductions_json = if advance_ids.is_empty() {
            None
        } else {
            Some(serde_json::json!({ "advance_ids": advance_ids }))
        };

        let carry_forward_balance_aed = {
            let prev = prev_month(existing.period_month);
            match self.repo.find_by_driver_month(existing.driver_id, prev).await? {
                Some(prev_salary) if prev_salary.net_payable_aed < Decimal::ZERO => prev_salary.net_payable_aed.abs(),
                _ => Decimal::ZERO,
            }
        };

        // 4. Derived totals
        let salik_aed = fields.salik_used_aed - fields.salik_refund_aed;
        let cash_submit_aed = fields.total_cash_received_aed - fields.cash_not_handover_aed;
        let car_charging_diff_aed = fields.car_charging_used_aed.map(|used| fields.car_charging_aed - used);
        let cash_diff_aed = fields.total_cash_submit_aed.map(|sub| fields.total_cash_received_aed - sub);

        // 5. Formula (same as generate)
        let (base_amount_aed, commission_aed, target_amount_aed, fixed_car_charging_aed, final_salary_aed) =
            match fields.salary_type {
                SalaryType::Commission => {
                    let base = fields.total_earnings_aed - fields.car_charging_aed - salik_aed;
                    let commission = base * commission_rate;
                    let final_sal = commission
                        + fields.incentives_aed
                        - fields.rta_fine_aed
                        - cash_submit_aed
                        - fields.card_service_charges_aed;
                    (base, Some(commission), None, None, final_sal)
                }
                SalaryType::TargetHigh => {
                    let car_adj = fixed_car_high_aed - fields.car_charging_used_aed.unwrap_or(Decimal::ZERO);
                    let cash_adj = fields.total_cash_received_aed - fields.cash_not_handover_aed;
                    let base = (fields.total_earnings_aed - target_high_aed) - car_adj - cash_adj;
                    let final_sal = base
                        + fields.incentives_aed
                        - fields.rta_fine_aed
                        - salik_aed
                        - fields.card_service_charges_aed
                        - effective_room_rent;
                    (base, None, Some(target_high_aed), Some(fixed_car_high_aed), final_sal)
                }
                SalaryType::TargetLow => {
                    let car_adj = fixed_car_low_aed - fields.car_charging_used_aed.unwrap_or(Decimal::ZERO);
                    let cash_adj = fields.total_cash_received_aed - fields.cash_not_handover_aed;
                    let base = (fields.total_earnings_aed - target_low_aed) - car_adj - cash_adj;
                    let final_sal = base
                        + fields.incentives_aed
                        - fields.rta_fine_aed
                        - salik_aed
                        - fields.card_service_charges_aed
                        - effective_room_rent;
                    (base, None, Some(target_low_aed), Some(fixed_car_low_aed), final_sal)
                }
            };

        let net_payable_aed = final_salary_aed - advance_deduction_aed - carry_forward_balance_aed;

        // 6. Build diff JSON
        let mut diff = serde_json::Map::new();
        macro_rules! diff_field {
            ($name:expr, $old:expr, $new:expr) => {
                if $old != $new {
                    diff.insert($name.to_string(), serde_json::json!({ "old": $old, "new": $new }));
                }
            };
        }
        diff_field!("salary_type", format!("{:?}", existing.salary_type_snapshot), format!("{:?}", fields.salary_type));
        diff_field!("total_earnings_aed", existing.total_earnings_aed, fields.total_earnings_aed);
        diff_field!("total_cash_received_aed", existing.total_cash_received_aed, fields.total_cash_received_aed);
        diff_field!("cash_not_handover_aed", existing.cash_not_handover_aed, fields.cash_not_handover_aed);
        diff_field!("car_charging_aed", existing.car_charging_aed, fields.car_charging_aed);
        diff_field!("salik_used_aed", existing.salik_used_aed, fields.salik_used_aed);
        diff_field!("salik_refund_aed", existing.salik_refund_aed, fields.salik_refund_aed);
        diff_field!("rta_fine_aed", existing.rta_fine_aed, fields.rta_fine_aed);
        diff_field!("card_service_charges_aed", existing.card_service_charges_aed, fields.card_service_charges_aed);
        diff_field!("net_payable_aed", existing.net_payable_aed, net_payable_aed);
        let diff_json = if diff.is_empty() { None } else { Some(serde_json::Value::Object(diff.clone())) };

        // 7. Build payload
        let payload = CreateSalary {
            driver_id: existing.driver_id,
            period_month: existing.period_month,
            salary_type_snapshot: fields.salary_type,
            total_earnings_aed: fields.total_earnings_aed,
            total_cash_received_aed: fields.total_cash_received_aed,
            total_cash_submit_aed: Some(cash_submit_aed),
            cash_not_handover_aed: fields.cash_not_handover_aed,
            cash_diff_aed,
            car_charging_aed: fields.car_charging_aed,
            car_charging_used_aed: fields.car_charging_used_aed,
            car_charging_diff_aed,
            salik_used_aed: fields.salik_used_aed,
            salik_refund_aed: fields.salik_refund_aed,
            salik_aed,
            rta_fine_aed: fields.rta_fine_aed,
            card_service_charges_aed: fields.card_service_charges_aed,
            room_rent_aed: if effective_room_rent > Decimal::ZERO { Some(effective_room_rent) } else { None },
            target_amount_aed,
            fixed_car_charging_aed,
            commission_aed,
            base_amount_aed,
            final_salary_aed,
            advance_deduction_aed,
            net_payable_aed,
            carry_forward_balance_aed,
            adjusted_from_id: Some(salary_id),
            deductions_json,
            generated_by: actor_id,
            incentives_aed: fields.incentives_aed,
        };

        let salary = self.repo.create_adjustment(salary_id, payload, diff_json).await?;

        // 8. Audit log
        self.audit.log(actor_id, actor_role, "salary", Some(salary_id), "salary.edited",
            Some(serde_json::json!({
                "salary_id": salary_id,
                "diff": serde_json::Value::Object(diff),
            }))).await?;

        Ok(salary)
    }

    pub async fn generate_slip(&self, salary_id: Uuid) -> Result<String, AppError> {
        let salary = self.repo.find_by_id(salary_id).await?;
        let url = self.pdf.generate_and_upload(&salary).await?;
        self.repo.update_slip_url(salary_id, &url).await?;
        Ok(url)
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
    pub incentives_aed:           Decimal,
}

pub struct EditSalaryFields {
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
    pub room_rent_aed:            Option<Decimal>,
    pub driver_room_rent_aed:     Decimal,
    pub driver_commission_rate:   Option<Decimal>,
    pub incentives_aed:           Decimal,
}

fn prev_month(d: NaiveDate) -> NaiveDate {
    if d.month() == 1 {
        NaiveDate::from_ymd_opt(d.year() - 1, 12, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(d.year(), d.month() - 1, 1).unwrap()
    }
}
