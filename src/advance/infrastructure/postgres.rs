use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{error::AppError, ports::DeductionPort};
use crate::advance::domain::{
    entity::{Advance, AdvanceStatus, ApproveAdvance, CreateAdvance, PayAdvance, PaymentMethod, RejectAdvance},
    repository::AdvanceRepository,
};

pub struct PgAdvanceRepository {
    pool: PgPool,
}

impl PgAdvanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdvanceRepository for PgAdvanceRepository {
    async fn list(
        &self,
        driver_id: Option<Uuid>,
        status: Option<AdvanceStatus>,
    ) -> Result<Vec<Advance>, AppError> {
        let rows = sqlx::query_as!(
            Advance,
            r#"
            SELECT
                a.id, a.driver_id,
                p.full_name AS driver_name,
                a.amount_aed,
                a.reason,
                a.status AS "status: AdvanceStatus",
                a.rejection_reason,
                a.payment_date,
                a.method AS "method: PaymentMethod",
                a.carry_forward_aed,
                a.salary_period,
                a.actioned_by,
                pa.full_name AS actioned_by_name,
                a.created_at,
                a.updated_at
            FROM advances a
            JOIN drivers d ON d.id = a.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = a.actioned_by
            WHERE ($1::uuid IS NULL OR a.driver_id = $1)
              AND ($2::advance_status IS NULL OR a.status = $2)
            ORDER BY a.created_at DESC
            "#,
            driver_id as Option<Uuid>,
            status as Option<AdvanceStatus>
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Advance, AppError> {
        let row = sqlx::query_as!(
            Advance,
            r#"
            SELECT
                a.id, a.driver_id,
                p.full_name AS driver_name,
                a.amount_aed,
                a.reason,
                a.status AS "status: AdvanceStatus",
                a.rejection_reason,
                a.payment_date,
                a.method AS "method: PaymentMethod",
                a.carry_forward_aed,
                a.salary_period,
                a.actioned_by,
                pa.full_name AS actioned_by_name,
                a.created_at,
                a.updated_at
            FROM advances a
            JOIN drivers d ON d.id = a.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = a.actioned_by
            WHERE a.id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Advance not found".into()))?;

        Ok(row)
    }

    async fn create(&self, payload: CreateAdvance) -> Result<Advance, AppError> {
        let row = sqlx::query_as!(
            Advance,
            r#"
            WITH ins AS (
                INSERT INTO advances (driver_id, amount_aed, reason)
                VALUES ($1, $2, $3)
                RETURNING *
            )
            SELECT
                ins.id, ins.driver_id,
                p.full_name AS driver_name,
                ins.amount_aed,
                ins.reason,
                ins.status AS "status: AdvanceStatus",
                ins.rejection_reason,
                ins.payment_date,
                ins.method AS "method: PaymentMethod",
                ins.carry_forward_aed,
                ins.salary_period,
                ins.actioned_by,
                pa.full_name AS actioned_by_name,
                ins.created_at,
                ins.updated_at
            FROM ins
            JOIN drivers d ON d.id = ins.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = ins.actioned_by
            "#,
            payload.driver_id,
            payload.amount_aed,
            payload.reason
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn approve(&self, payload: ApproveAdvance) -> Result<Advance, AppError> {
        let row = sqlx::query_as!(
            Advance,
            r#"
            WITH upd AS (
                UPDATE advances
                SET status = 'approved', actioned_by = $2, updated_at = NOW()
                WHERE id = $1 AND status = 'pending'
                RETURNING *
            )
            SELECT
                upd.id, upd.driver_id,
                p.full_name AS driver_name,
                upd.amount_aed,
                upd.reason,
                upd.status AS "status: AdvanceStatus",
                upd.rejection_reason,
                upd.payment_date,
                upd.method AS "method: PaymentMethod",
                upd.carry_forward_aed,
                upd.salary_period,
                upd.actioned_by,
                pa.full_name AS actioned_by_name,
                upd.created_at,
                upd.updated_at
            FROM upd
            JOIN drivers d ON d.id = upd.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = upd.actioned_by
            "#,
            payload.id,
            payload.actioned_by
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::BadRequest("Advance not found or not in pending state".into()))?;

        Ok(row)
    }

    async fn reject(&self, payload: RejectAdvance) -> Result<Advance, AppError> {
        let row = sqlx::query_as!(
            Advance,
            r#"
            WITH upd AS (
                UPDATE advances
                SET status = 'rejected',
                    rejection_reason = $3,
                    actioned_by = $2,
                    updated_at = NOW()
                WHERE id = $1 AND status = 'pending'
                RETURNING *
            )
            SELECT
                upd.id, upd.driver_id,
                p.full_name AS driver_name,
                upd.amount_aed,
                upd.reason,
                upd.status AS "status: AdvanceStatus",
                upd.rejection_reason,
                upd.payment_date,
                upd.method AS "method: PaymentMethod",
                upd.carry_forward_aed,
                upd.salary_period,
                upd.actioned_by,
                pa.full_name AS actioned_by_name,
                upd.created_at,
                upd.updated_at
            FROM upd
            JOIN drivers d ON d.id = upd.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = upd.actioned_by
            "#,
            payload.id,
            payload.actioned_by,
            payload.rejection_reason
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::BadRequest("Advance not found or not in pending state".into()))?;

        Ok(row)
    }

    async fn pay(&self, payload: PayAdvance) -> Result<Advance, AppError> {
        let row = sqlx::query_as!(
            Advance,
            r#"
            WITH upd AS (
                UPDATE advances
                SET status = 'paid',
                    payment_date = $3,
                    method = $4,
                    salary_period = $5,
                    actioned_by = $2,
                    updated_at = NOW()
                WHERE id = $1 AND status = 'approved'
                RETURNING *
            )
            SELECT
                upd.id, upd.driver_id,
                p.full_name AS driver_name,
                upd.amount_aed,
                upd.reason,
                upd.status AS "status: AdvanceStatus",
                upd.rejection_reason,
                upd.payment_date,
                upd.method AS "method: PaymentMethod",
                upd.carry_forward_aed,
                upd.salary_period,
                upd.actioned_by,
                pa.full_name AS actioned_by_name,
                upd.created_at,
                upd.updated_at
            FROM upd
            JOIN drivers d ON d.id = upd.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = upd.actioned_by
            "#,
            payload.id,
            payload.actioned_by,
            payload.payment_date,
            payload.method as PaymentMethod,
            payload.salary_period
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::BadRequest("Advance not found or not in approved state".into()))?;

        Ok(row)
    }

    async fn count_pending(&self, driver_id: Uuid) -> Result<i64, AppError> {
        let row = sqlx::query!(
            "SELECT COUNT(*) AS count FROM advances WHERE driver_id = $1 AND status = 'pending'",
            driver_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0))
    }
}

// ── DeductionPort implementation ────────────────────────────────────────────
// Salary module consumes this to get advance deductions without importing advance.

#[async_trait]
impl DeductionPort for PgAdvanceRepository {
    async fn get_advance_deductions(
        &self,
        driver_id: Uuid,
        period_month: NaiveDate,
    ) -> Result<Decimal, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount_aed + carry_forward_aed), 0) AS total
            FROM advances
            WHERE driver_id = $1
              AND status = 'paid'
              AND salary_period = $2
            "#,
            driver_id,
            period_month
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.total.unwrap_or(Decimal::ZERO))
    }

    async fn get_advance_ids_for_period(
        &self,
        driver_id: Uuid,
        period_month: NaiveDate,
    ) -> Result<Vec<Uuid>, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT id FROM advances
            WHERE driver_id = $1
              AND status = 'paid'
              AND salary_period = $2
            "#,
            driver_id,
            period_month
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.id).collect())
    }
}
