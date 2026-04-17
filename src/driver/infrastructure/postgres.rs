use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use rust_decimal::Decimal;
use crate::common::{error::AppError, types::SalaryType};
use crate::driver::domain::{
    entity::{Driver, DriverEdit},
    repository::DriverRepository,
};

pub struct PgDriverRepository {
    pool: sqlx::PgPool,
}

impl PgDriverRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DriverRepository for PgDriverRepository {
    async fn list(&self) -> Result<Vec<Driver>, AppError> {
        let rows = sqlx::query_as!(
            Driver,
            r#"SELECT d.id, d.profile_id, p.full_name, p.email,
                      d.nationality, d.salary_type as "salary_type: SalaryType",
                      d.is_active, d.self_entry_enabled,
                      d.room_rent_aed, d.commission_rate,
                      d.joining_date,
                      d.created_at
               FROM drivers d
               JOIN profiles p ON p.id = d.profile_id
               ORDER BY p.full_name"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Driver>, AppError> {
        let row = sqlx::query_as!(
            Driver,
            r#"SELECT d.id, d.profile_id, p.full_name, p.email,
                      d.nationality, d.salary_type as "salary_type: SalaryType",
                      d.is_active, d.self_entry_enabled,
                      d.room_rent_aed, d.commission_rate,
                      d.joining_date,
                      d.created_at
               FROM drivers d
               JOIN profiles p ON p.id = d.profile_id
               WHERE d.id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_by_profile_id(&self, profile_id: Uuid) -> Result<Option<Driver>, AppError> {
        let row = sqlx::query_as!(
            Driver,
            r#"SELECT d.id, d.profile_id, p.full_name, p.email,
                      d.nationality, d.salary_type as "salary_type: SalaryType",
                      d.is_active, d.self_entry_enabled,
                      d.room_rent_aed, d.commission_rate,
                      d.joining_date,
                      d.created_at
               FROM drivers d
               JOIN profiles p ON p.id = d.profile_id
               WHERE d.profile_id = $1"#,
            profile_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create(&self, profile_id: Uuid, nationality: &str, salary_type: SalaryType, room_rent_aed: Decimal, commission_rate: Option<Decimal>, joining_date: Option<NaiveDate>) -> Result<Driver, AppError> {
        let row = sqlx::query_as!(
            Driver,
            r#"WITH ins AS (
                INSERT INTO drivers (profile_id, nationality, salary_type, room_rent_aed, commission_rate, joining_date)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id, profile_id, nationality, salary_type, is_active, self_entry_enabled, room_rent_aed, commission_rate, joining_date, created_at
               )
               SELECT ins.id, ins.profile_id, p.full_name, p.email,
                      ins.nationality, ins.salary_type as "salary_type: SalaryType",
                      ins.is_active, ins.self_entry_enabled,
                      ins.room_rent_aed, ins.commission_rate,
                      ins.joining_date,
                      ins.created_at
               FROM ins JOIN profiles p ON p.id = ins.profile_id"#,
            profile_id,
            nationality,
            salary_type as SalaryType,
            room_rent_aed,
            commission_rate,
            joining_date,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn update(&self, id: Uuid, nationality: &str, salary_type: SalaryType, room_rent_aed: Decimal, commission_rate: Option<Decimal>, joining_date: Option<NaiveDate>) -> Result<Driver, AppError> {
        let row = sqlx::query_as!(
            Driver,
            r#"WITH upd AS (
                UPDATE drivers SET nationality = $2, salary_type = $3, room_rent_aed = $4, commission_rate = $5, joining_date = $6, updated_at = NOW()
                WHERE id = $1
                RETURNING id, profile_id, nationality, salary_type, is_active, self_entry_enabled, room_rent_aed, commission_rate, joining_date, created_at
               )
               SELECT upd.id, upd.profile_id, p.full_name, p.email,
                      upd.nationality, upd.salary_type as "salary_type: SalaryType",
                      upd.is_active, upd.self_entry_enabled,
                      upd.room_rent_aed, upd.commission_rate,
                      upd.joining_date,
                      upd.created_at
               FROM upd JOIN profiles p ON p.id = upd.profile_id"#,
            id,
            nationality,
            salary_type as SalaryType,
            room_rent_aed,
            commission_rate,
            joining_date,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn set_active(&self, id: Uuid, active: bool) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE drivers SET is_active = $1, updated_at = NOW() WHERE id = $2",
            active, id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn has_active_vehicle(&self, id: Uuid) -> Result<bool, AppError> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM vehicle_assignments WHERE driver_id = $1 AND unassigned_at IS NULL)"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    async fn log_edit(&self, driver_id: Uuid, changed_by: Uuid, field: &str, old_val: Option<&str>, new_val: Option<&str>) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO driver_edits (driver_id, changed_by, field, old_val, new_val) VALUES ($1, $2, $3, $4, $5)",
            driver_id, changed_by, field, old_val, new_val
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_edits(&self, driver_id: Uuid) -> Result<Vec<DriverEdit>, AppError> {
        let rows = sqlx::query_as!(
            DriverEdit,
            "SELECT id, driver_id, changed_by, field, old_val, new_val, changed_at FROM driver_edits WHERE driver_id = $1 ORDER BY changed_at DESC",
            driver_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn set_self_entry(&self, id: Uuid, enabled: bool) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE drivers SET self_entry_enabled = $1, updated_at = NOW() WHERE id = $2",
            enabled, id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
