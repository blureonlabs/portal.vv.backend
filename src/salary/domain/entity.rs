use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::common::types::SalaryType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "salary_status", rename_all = "snake_case")]
pub enum SalaryStatus {
    Draft,
    Approved,
    Paid,
}

#[derive(Debug, Clone)]
pub struct Salary {
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
    pub deductions_json: Option<Value>,
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

pub struct CreateSalary {
    pub driver_id: Uuid,
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
    pub deductions_json: Option<Value>,
    pub generated_by: Uuid,
}
