use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::types::SalaryType;
use crate::salary::domain::entity::{Salary, SalaryStatus};

#[derive(Debug, Deserialize)]
pub struct GenerateSalaryBody {
    pub driver_id: Uuid,
    /// "YYYY-MM" — first day of month is used as period_month
    pub period_month: String,
    pub salary_type: SalaryType,
    pub total_earnings_aed: Decimal,
    pub total_cash_received_aed: Decimal,
    pub total_cash_submit_aed: Option<Decimal>,
    #[serde(default)]
    pub cash_not_handover_aed: Decimal,
    #[serde(default)]
    pub car_charging_aed: Decimal,
    pub car_charging_used_aed: Option<Decimal>,
    #[serde(default)]
    pub salik_used_aed: Decimal,
    #[serde(default)]
    pub salik_refund_aed: Decimal,
    #[serde(default)]
    pub rta_fine_aed: Decimal,
    #[serde(default)]
    pub card_service_charges_aed: Decimal,
    pub room_rent_aed: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct ListSalaryQuery {
    pub driver_id: Option<Uuid>,
    /// "YYYY-MM"
    pub month: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FetchEarningsQuery {
    pub driver_id: Uuid,
    /// "YYYY-MM"
    pub month: String,
}

#[derive(Debug, Deserialize)]
pub struct MarkPaidRequest {
    pub payment_date: NaiveDate,
    pub payment_mode: String,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SalaryResponse {
    pub id: Uuid,
    pub driver_id: Uuid,
    pub driver_name: String,
    pub period_month: NaiveDate,
    pub salary_type_snapshot: SalaryType,
    pub total_earnings_aed: Decimal,
    pub total_cash_received_aed: Decimal,
    pub total_cash_submit_aed: Option<Decimal>,
    pub cash_not_handover_aed: Decimal,
    pub cash_diff_aed: Option<Decimal>,
    pub car_charging_aed: Decimal,
    pub car_charging_used_aed: Option<Decimal>,
    pub car_charging_diff_aed: Option<Decimal>,
    pub salik_used_aed: Decimal,
    pub salik_refund_aed: Decimal,
    pub salik_aed: Decimal,
    pub rta_fine_aed: Decimal,
    pub card_service_charges_aed: Decimal,
    pub room_rent_aed: Option<Decimal>,
    pub target_amount_aed: Option<Decimal>,
    pub fixed_car_charging_aed: Option<Decimal>,
    pub commission_aed: Option<Decimal>,
    pub base_amount_aed: Decimal,
    pub final_salary_aed: Decimal,
    pub advance_deduction_aed: Decimal,
    pub net_payable_aed: Decimal,
    pub deductions_json: Option<serde_json::Value>,
    pub slip_url: Option<String>,
    pub generated_by: Uuid,
    pub generated_by_name: String,
    pub generated_at: DateTime<Utc>,
    pub status: SalaryStatus,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub payment_date: Option<NaiveDate>,
    pub payment_mode: Option<String>,
    pub payment_reference: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
}

impl From<Salary> for SalaryResponse {
    fn from(s: Salary) -> Self {
        Self {
            id: s.id,
            driver_id: s.driver_id,
            driver_name: s.driver_name,
            period_month: s.period_month,
            salary_type_snapshot: s.salary_type_snapshot,
            total_earnings_aed: s.total_earnings_aed,
            total_cash_received_aed: s.total_cash_received_aed,
            total_cash_submit_aed: s.total_cash_submit_aed,
            cash_not_handover_aed: s.cash_not_handover_aed,
            cash_diff_aed: s.cash_diff_aed,
            car_charging_aed: s.car_charging_aed,
            car_charging_used_aed: s.car_charging_used_aed,
            car_charging_diff_aed: s.car_charging_diff_aed,
            salik_used_aed: s.salik_used_aed,
            salik_refund_aed: s.salik_refund_aed,
            salik_aed: s.salik_aed,
            rta_fine_aed: s.rta_fine_aed,
            card_service_charges_aed: s.card_service_charges_aed,
            room_rent_aed: s.room_rent_aed,
            target_amount_aed: s.target_amount_aed,
            fixed_car_charging_aed: s.fixed_car_charging_aed,
            commission_aed: s.commission_aed,
            base_amount_aed: s.base_amount_aed,
            final_salary_aed: s.final_salary_aed,
            advance_deduction_aed: s.advance_deduction_aed,
            net_payable_aed: s.net_payable_aed,
            deductions_json: s.deductions_json,
            slip_url: s.slip_url,
            generated_by: s.generated_by,
            generated_by_name: s.generated_by_name,
            generated_at: s.generated_at,
            status: s.status,
            approved_by: s.approved_by,
            approved_at: s.approved_at,
            payment_date: s.payment_date,
            payment_mode: s.payment_mode,
            payment_reference: s.payment_reference,
            paid_at: s.paid_at,
        }
    }
}
