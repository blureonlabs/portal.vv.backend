use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::hr::domain::{
    entity::{ActionLeaveRequest, CreateLeaveRequest, LeaveRequest, LeaveStatus, LeaveType},
    repository::HrRepository,
};

pub struct PgHrRepository {
    pool: PgPool,
}

impl PgHrRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HrRepository for PgHrRepository {
    async fn list(
        &self,
        driver_id: Option<Uuid>,
        status: Option<LeaveStatus>,
        leave_type: Option<LeaveType>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        let rows = sqlx::query_as!(
            LeaveRequest,
            r#"
            SELECT
                lr.id, lr.driver_id,
                p.full_name AS driver_name,
                lr.type AS "type: LeaveType",
                lr.from_date, lr.to_date, lr.reason,
                lr.status AS "status: LeaveStatus",
                lr.actioned_by,
                pa.full_name AS actioned_by_name,
                lr.rejection_reason,
                lr.created_at, lr.updated_at
            FROM leave_requests lr
            JOIN drivers d ON d.id = lr.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = lr.actioned_by
            WHERE ($1::uuid IS NULL OR lr.driver_id = $1)
              AND ($2::leave_status IS NULL OR lr.status = $2)
              AND ($3::leave_type IS NULL OR lr.type = $3)
            ORDER BY lr.created_at DESC
            "#,
            driver_id as Option<Uuid>,
            status as Option<LeaveStatus>,
            leave_type as Option<LeaveType>
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<LeaveRequest, AppError> {
        let row = sqlx::query_as!(
            LeaveRequest,
            r#"
            SELECT
                lr.id, lr.driver_id,
                p.full_name AS driver_name,
                lr.type AS "type: LeaveType",
                lr.from_date, lr.to_date, lr.reason,
                lr.status AS "status: LeaveStatus",
                lr.actioned_by,
                pa.full_name AS actioned_by_name,
                lr.rejection_reason,
                lr.created_at, lr.updated_at
            FROM leave_requests lr
            JOIN drivers d ON d.id = lr.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = lr.actioned_by
            WHERE lr.id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Leave request not found".into()))?;

        Ok(row)
    }

    async fn create(&self, payload: CreateLeaveRequest) -> Result<LeaveRequest, AppError> {
        let row = sqlx::query_as!(
            LeaveRequest,
            r#"
            WITH ins AS (
                INSERT INTO leave_requests (driver_id, type, from_date, to_date, reason)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
            )
            SELECT
                ins.id, ins.driver_id,
                p.full_name AS driver_name,
                ins.type AS "type: LeaveType",
                ins.from_date, ins.to_date, ins.reason,
                ins.status AS "status: LeaveStatus",
                ins.actioned_by,
                pa.full_name AS actioned_by_name,
                ins.rejection_reason,
                ins.created_at, ins.updated_at
            FROM ins
            JOIN drivers d ON d.id = ins.driver_id
            JOIN profiles p ON p.id = d.profile_id
            LEFT JOIN profiles pa ON pa.id = ins.actioned_by
            "#,
            payload.driver_id,
            payload.r#type as LeaveType,
            payload.from_date,
            payload.to_date,
            payload.reason
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn approve(&self, payload: ActionLeaveRequest) -> Result<LeaveRequest, AppError> {
        let row = sqlx::query_as!(
            LeaveRequest,
            r#"
            WITH upd AS (
                UPDATE leave_requests
                SET status = 'approved', actioned_by = $2, updated_at = NOW()
                WHERE id = $1 AND status = 'pending'
                RETURNING *
            )
            SELECT
                upd.id, upd.driver_id,
                p.full_name AS driver_name,
                upd.type AS "type: LeaveType",
                upd.from_date, upd.to_date, upd.reason,
                upd.status AS "status: LeaveStatus",
                upd.actioned_by,
                pa.full_name AS actioned_by_name,
                upd.rejection_reason,
                upd.created_at, upd.updated_at
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
        .ok_or_else(|| AppError::BadRequest("Leave request not found or not in pending state".into()))?;

        Ok(row)
    }

    async fn reject(&self, payload: ActionLeaveRequest) -> Result<LeaveRequest, AppError> {
        let row = sqlx::query_as!(
            LeaveRequest,
            r#"
            WITH upd AS (
                UPDATE leave_requests
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
                upd.type AS "type: LeaveType",
                upd.from_date, upd.to_date, upd.reason,
                upd.status AS "status: LeaveStatus",
                upd.actioned_by,
                pa.full_name AS actioned_by_name,
                upd.rejection_reason,
                upd.created_at, upd.updated_at
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
        .ok_or_else(|| AppError::BadRequest("Leave request not found or not in pending state".into()))?;

        Ok(row)
    }

    async fn count_overlapping_approved(
        &self,
        driver_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
        exclude_id: Option<Uuid>,
    ) -> Result<i64, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS count FROM leave_requests
            WHERE driver_id = $1
              AND status = 'approved'
              AND from_date <= $3
              AND to_date >= $2
              AND ($4::uuid IS NULL OR id != $4)
            "#,
            driver_id,
            from_date,
            to_date,
            exclude_id as Option<Uuid>
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0))
    }

    async fn bulk_approve(&self, request_ids: &[Uuid], actioned_by: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE leave_requests
            SET status = 'approved', actioned_by = $2, updated_at = NOW()
            WHERE id = ANY($1)
              AND status = 'pending'
            "#,
            request_ids,
            actioned_by
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
