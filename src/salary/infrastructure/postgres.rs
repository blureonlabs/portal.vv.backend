use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{error::AppError, types::SalaryType};
use crate::salary::domain::{entity::{CreateSalary, Salary}, repository::SalaryRepository};

pub struct PgSalaryRepository {
    pool: PgPool,
}

impl PgSalaryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SalaryRow {
    id: Uuid,
    driver_id: Uuid,
    driver_name: String,
    period_month: NaiveDate,
    salary_type_snapshot: SalaryType,
    total_earnings_aed: Decimal,
    total_cash_received_aed: Decimal,
    total_cash_submit_aed: Option<Decimal>,
    cash_not_handover_aed: Decimal,
    cash_diff_aed: Option<Decimal>,
    car_charging_aed: Decimal,
    car_charging_used_aed: Option<Decimal>,
    car_charging_diff_aed: Option<Decimal>,
    salik_used_aed: Decimal,
    salik_refund_aed: Decimal,
    salik_aed: Decimal,
    rta_fine_aed: Decimal,
    card_service_charges_aed: Decimal,
    room_rent_aed: Option<Decimal>,
    target_amount_aed: Option<Decimal>,
    fixed_car_charging_aed: Option<Decimal>,
    commission_aed: Option<Decimal>,
    base_amount_aed: Decimal,
    final_salary_aed: Decimal,
    advance_deduction_aed: Decimal,
    net_payable_aed: Decimal,
    deductions_json: Option<serde_json::Value>,
    slip_url: Option<String>,
    generated_by: Uuid,
    generated_by_name: String,
    generated_at: chrono::DateTime<chrono::Utc>,
}

fn row_to_salary(r: SalaryRow) -> Salary {
    Salary {
        id: r.id,
        driver_id: r.driver_id,
        driver_name: r.driver_name,
        period_month: r.period_month,
        salary_type_snapshot: r.salary_type_snapshot,
        total_earnings_aed: r.total_earnings_aed,
        total_cash_received_aed: r.total_cash_received_aed,
        total_cash_submit_aed: r.total_cash_submit_aed,
        cash_not_handover_aed: r.cash_not_handover_aed,
        cash_diff_aed: r.cash_diff_aed,
        car_charging_aed: r.car_charging_aed,
        car_charging_used_aed: r.car_charging_used_aed,
        car_charging_diff_aed: r.car_charging_diff_aed,
        salik_used_aed: r.salik_used_aed,
        salik_refund_aed: r.salik_refund_aed,
        salik_aed: r.salik_aed,
        rta_fine_aed: r.rta_fine_aed,
        card_service_charges_aed: r.card_service_charges_aed,
        room_rent_aed: r.room_rent_aed,
        target_amount_aed: r.target_amount_aed,
        fixed_car_charging_aed: r.fixed_car_charging_aed,
        commission_aed: r.commission_aed,
        base_amount_aed: r.base_amount_aed,
        final_salary_aed: r.final_salary_aed,
        advance_deduction_aed: r.advance_deduction_aed,
        net_payable_aed: r.net_payable_aed,
        deductions_json: r.deductions_json,
        slip_url: r.slip_url,
        generated_by: r.generated_by,
        generated_by_name: r.generated_by_name,
        generated_at: r.generated_at,
    }
}

const SELECT_FIELDS: &str = r#"
    s.id, s.driver_id, pd.full_name AS driver_name, s.period_month,
    s.salary_type_snapshot,
    s.total_earnings_aed, s.total_cash_received_aed, s.total_cash_submit_aed,
    s.cash_not_handover_aed, s.cash_diff_aed,
    s.car_charging_aed, s.car_charging_used_aed, s.car_charging_diff_aed,
    s.salik_used_aed, s.salik_refund_aed, s.salik_aed,
    s.rta_fine_aed, s.card_service_charges_aed, s.room_rent_aed,
    s.target_amount_aed, s.fixed_car_charging_aed, s.commission_aed,
    s.base_amount_aed, s.final_salary_aed, s.advance_deduction_aed, s.net_payable_aed,
    s.deductions_json, s.slip_url,
    s.generated_by, pg.full_name AS generated_by_name, s.generated_at
"#;

#[async_trait]
impl SalaryRepository for PgSalaryRepository {
    async fn list(&self, driver_id: Option<Uuid>, month: Option<NaiveDate>) -> Result<Vec<Salary>, AppError> {
        let rows = sqlx::query_as::<_, SalaryRow>(&format!(
            "SELECT {} FROM salaries s \
             JOIN drivers d ON d.id = s.driver_id \
             JOIN profiles pd ON pd.id = d.profile_id \
             JOIN profiles pg ON pg.id = s.generated_by \
             WHERE ($1::uuid IS NULL OR s.driver_id = $1) \
               AND ($2::date IS NULL OR s.period_month = $2) \
             ORDER BY s.period_month DESC, pd.full_name",
            SELECT_FIELDS
        ))
        .bind(driver_id)
        .bind(month)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(row_to_salary).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Salary, AppError> {
        let row = sqlx::query_as::<_, SalaryRow>(&format!(
            "SELECT {} FROM salaries s \
             JOIN drivers d ON d.id = s.driver_id \
             JOIN profiles pd ON pd.id = d.profile_id \
             JOIN profiles pg ON pg.id = s.generated_by \
             WHERE s.id = $1",
            SELECT_FIELDS
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Salary {id}")))?;

        Ok(row_to_salary(row))
    }

    async fn find_by_driver_month(&self, driver_id: Uuid, period_month: NaiveDate) -> Result<Option<Salary>, AppError> {
        let row = sqlx::query_as::<_, SalaryRow>(&format!(
            "SELECT {} FROM salaries s \
             JOIN drivers d ON d.id = s.driver_id \
             JOIN profiles pd ON pd.id = d.profile_id \
             JOIN profiles pg ON pg.id = s.generated_by \
             WHERE s.driver_id = $1 AND s.period_month = $2",
            SELECT_FIELDS
        ))
        .bind(driver_id)
        .bind(period_month)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(row_to_salary))
    }

    async fn upsert(&self, p: CreateSalary) -> Result<Salary, AppError> {
        let row = sqlx::query_as::<_, SalaryRow>(&format!(
            r#"
            INSERT INTO salaries (
                driver_id, period_month, salary_type_snapshot,
                total_earnings_aed, total_cash_received_aed, total_cash_submit_aed,
                cash_not_handover_aed, cash_diff_aed,
                car_charging_aed, car_charging_used_aed, car_charging_diff_aed,
                salik_used_aed, salik_refund_aed, salik_aed,
                rta_fine_aed, card_service_charges_aed, room_rent_aed,
                target_amount_aed, fixed_car_charging_aed, commission_aed,
                base_amount_aed, final_salary_aed, advance_deduction_aed, net_payable_aed,
                deductions_json, generated_by
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26)
            ON CONFLICT (driver_id, period_month) DO UPDATE SET
                salary_type_snapshot      = EXCLUDED.salary_type_snapshot,
                total_earnings_aed        = EXCLUDED.total_earnings_aed,
                total_cash_received_aed   = EXCLUDED.total_cash_received_aed,
                total_cash_submit_aed     = EXCLUDED.total_cash_submit_aed,
                cash_not_handover_aed     = EXCLUDED.cash_not_handover_aed,
                cash_diff_aed             = EXCLUDED.cash_diff_aed,
                car_charging_aed          = EXCLUDED.car_charging_aed,
                car_charging_used_aed     = EXCLUDED.car_charging_used_aed,
                car_charging_diff_aed     = EXCLUDED.car_charging_diff_aed,
                salik_used_aed            = EXCLUDED.salik_used_aed,
                salik_refund_aed          = EXCLUDED.salik_refund_aed,
                salik_aed                 = EXCLUDED.salik_aed,
                rta_fine_aed              = EXCLUDED.rta_fine_aed,
                card_service_charges_aed  = EXCLUDED.card_service_charges_aed,
                room_rent_aed             = EXCLUDED.room_rent_aed,
                target_amount_aed         = EXCLUDED.target_amount_aed,
                fixed_car_charging_aed    = EXCLUDED.fixed_car_charging_aed,
                commission_aed            = EXCLUDED.commission_aed,
                base_amount_aed           = EXCLUDED.base_amount_aed,
                final_salary_aed          = EXCLUDED.final_salary_aed,
                advance_deduction_aed     = EXCLUDED.advance_deduction_aed,
                net_payable_aed           = EXCLUDED.net_payable_aed,
                deductions_json           = EXCLUDED.deductions_json,
                generated_by              = EXCLUDED.generated_by,
                generated_at              = NOW()
            RETURNING id, driver_id,
                (SELECT full_name FROM profiles WHERE id = (SELECT profile_id FROM drivers WHERE id = salaries.driver_id)) AS driver_name,
                period_month, salary_type_snapshot,
                total_earnings_aed, total_cash_received_aed, total_cash_submit_aed,
                cash_not_handover_aed, cash_diff_aed,
                car_charging_aed, car_charging_used_aed, car_charging_diff_aed,
                salik_used_aed, salik_refund_aed, salik_aed,
                rta_fine_aed, card_service_charges_aed, room_rent_aed,
                target_amount_aed, fixed_car_charging_aed, commission_aed,
                base_amount_aed, final_salary_aed, advance_deduction_aed, net_payable_aed,
                deductions_json, slip_url,
                generated_by,
                (SELECT full_name FROM profiles WHERE id = salaries.generated_by) AS generated_by_name,
                generated_at
            "#,
        ))
        .bind(p.driver_id)
        .bind(p.period_month)
        .bind(p.salary_type_snapshot as SalaryType)
        .bind(p.total_earnings_aed)
        .bind(p.total_cash_received_aed)
        .bind(p.total_cash_submit_aed)
        .bind(p.cash_not_handover_aed)
        .bind(p.cash_diff_aed)
        .bind(p.car_charging_aed)
        .bind(p.car_charging_used_aed)
        .bind(p.car_charging_diff_aed)
        .bind(p.salik_used_aed)
        .bind(p.salik_refund_aed)
        .bind(p.salik_aed)
        .bind(p.rta_fine_aed)
        .bind(p.card_service_charges_aed)
        .bind(p.room_rent_aed)
        .bind(p.target_amount_aed)
        .bind(p.fixed_car_charging_aed)
        .bind(p.commission_aed)
        .bind(p.base_amount_aed)
        .bind(p.final_salary_aed)
        .bind(p.advance_deduction_aed)
        .bind(p.net_payable_aed)
        .bind(p.deductions_json)
        .bind(p.generated_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(row_to_salary(row))
    }

    async fn update_slip_url(&self, id: Uuid, slip_url: &str) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE salaries SET slip_url = $2 WHERE id = $1",
            id,
            slip_url
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
