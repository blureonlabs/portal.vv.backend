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
    ) -> Result<Vec<Expense>, AppError> {
        let rows = sqlx::query_as!(
            Expense,
            r#"
            SELECT
                e.id, e.driver_id,
                p.full_name AS driver_name,
                e.entered_by,
                e.amount_aed,
                e.category AS "category: ExpenseCategory",
                e.date,
                e.receipt_url, e.notes, e.created_at
            FROM expenses e
            LEFT JOIN drivers d ON d.id = e.driver_id
            LEFT JOIN profiles p ON p.id = d.profile_id
            WHERE e.date BETWEEN $2 AND $3
              AND ($1::uuid IS NULL OR e.driver_id = $1)
            ORDER BY e.date DESC, e.created_at DESC
            "#,
            driver_id as Option<Uuid>,
            from,
            to
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn create_expense(&self, payload: CreateExpense) -> Result<Expense, AppError> {
        let row = sqlx::query_as!(
            Expense,
            r#"
            WITH ins AS (
                INSERT INTO expenses (driver_id, entered_by, amount_aed, category, date, receipt_url, notes)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING *
            )
            SELECT
                ins.id, ins.driver_id,
                p.full_name AS driver_name,
                ins.entered_by,
                ins.amount_aed,
                ins.category AS "category: ExpenseCategory",
                ins.date,
                ins.receipt_url, ins.notes, ins.created_at
            FROM ins
            LEFT JOIN drivers d ON d.id = ins.driver_id
            LEFT JOIN profiles p ON p.id = d.profile_id
            "#,
            payload.driver_id,
            payload.entered_by,
            payload.amount_aed,
            payload.category as ExpenseCategory,
            payload.date,
            payload.receipt_url,
            payload.notes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn list_handovers(
        &self,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<CashHandover>, AppError> {
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
            "#,
            driver_id as Option<Uuid>,
            from,
            to
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
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
