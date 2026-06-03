use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::finance::domain::{
    entity::{CashHandover, CreateExpense, CreateHandover, Expense, ExpenseCategory},
    repository::FinanceRepository,
};

pub struct PgFinanceRepository {
    pool: PgPool,
}

impl PgFinanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FinanceRepository for PgFinanceRepository {
    async fn list_expenses(
        &self,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Expense>, i64), AppError> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM expenses e \
             WHERE e.date BETWEEN $1 AND $2 \
               AND ($3::uuid IS NULL OR e.driver_id = $3)"
        )
        .bind(from)
        .bind(to)
        .bind(driver_id)
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query_as::<_, Expense>(
            "SELECT \
                e.id, e.driver_id, \
                p.full_name AS driver_name, \
                e.entered_by, \
                e.amount_aed, \
                e.category, \
                e.date, \
                e.receipt_url, e.notes, e.created_at \
            FROM expenses e \
            LEFT JOIN drivers d ON d.id = e.driver_id \
            LEFT JOIN profiles p ON p.id = d.profile_id \
            WHERE e.date BETWEEN $2 AND $3 \
              AND ($1::uuid IS NULL OR e.driver_id = $1) \
            ORDER BY e.date DESC, e.created_at DESC \
            LIMIT $4 OFFSET $5"
        )
        .bind(driver_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((rows, total.0))
    }

    async fn create_expense(&self, payload: CreateExpense) -> Result<Expense, AppError> {
        let row = sqlx::query_as::<_, Expense>(
            "WITH ins AS ( \
                INSERT INTO expenses (driver_id, entered_by, amount_aed, category, date, receipt_url, notes) \
                VALUES ($1, $2, $3, $4, $5, $6, $7) \
                RETURNING * \
            ) \
            SELECT \
                ins.id, ins.driver_id, \
                p.full_name AS driver_name, \
                ins.entered_by, \
                ins.amount_aed, \
                ins.category, \
                ins.date, \
                ins.receipt_url, ins.notes, ins.created_at \
            FROM ins \
            LEFT JOIN drivers d ON d.id = ins.driver_id \
            LEFT JOIN profiles p ON p.id = d.profile_id"
        )
        .bind(payload.driver_id)
        .bind(payload.entered_by)
        .bind(payload.amount_aed)
        .bind(payload.category as ExpenseCategory)
        .bind(payload.date)
        .bind(payload.receipt_url)
        .bind(payload.notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list_handovers(
        &self,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<CashHandover>, i64), AppError> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM cash_handovers h \
             WHERE h.submitted_at::date BETWEEN $1 AND $2 \
               AND ($3::uuid IS NULL OR h.driver_id = $3)"
        )
        .bind(from)
        .bind(to)
        .bind(driver_id)
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query_as!(
            CashHandover,
            r#"
            SELECT
                h.id, h.driver_id,
                pd.full_name AS driver_name,
                h.amount_aed,
                h.submitted_at,
                h.verified_by,
                pv.full_name AS verifier_name
            FROM cash_handovers h
            JOIN drivers d ON d.id = h.driver_id
            JOIN profiles pd ON pd.id = d.profile_id
            JOIN profiles pv ON pv.id = h.verified_by
            WHERE h.submitted_at::date BETWEEN $2 AND $3
              AND ($1::uuid IS NULL OR h.driver_id = $1)
            ORDER BY h.submitted_at DESC
            LIMIT $4 OFFSET $5
            "#,
            driver_id as Option<Uuid>,
            from,
            to,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok((rows, total.0))
    }

    async fn create_handover(&self, payload: CreateHandover) -> Result<CashHandover, AppError> {
        let row = sqlx::query_as!(
            CashHandover,
            r#"
            WITH ins AS (
                INSERT INTO cash_handovers (driver_id, amount_aed, verified_by)
                VALUES ($1, $2, $3)
                RETURNING *
            )
            SELECT
                ins.id, ins.driver_id,
                pd.full_name AS driver_name,
                ins.amount_aed,
                ins.submitted_at,
                ins.verified_by,
                pv.full_name AS verifier_name
            FROM ins
            JOIN drivers d ON d.id = ins.driver_id
            JOIN profiles pd ON pd.id = d.profile_id
            JOIN profiles pv ON pv.id = ins.verified_by
            "#,
            payload.driver_id,
            payload.amount_aed,
            payload.verified_by
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }
}
