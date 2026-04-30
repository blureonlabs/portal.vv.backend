use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::trip::domain::{
    entity::{CreateTrip, MonthlyEarnings, Trip, TripSource},
    repository::TripRepository,
};

pub struct PgTripRepository {
    pool: PgPool,
}

impl PgTripRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TripRepository for PgTripRepository {
    async fn list(
        &self,
        driver_id: Option<Uuid>,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Trip>, AppError> {
        let rows = sqlx::query_as!(
            Trip,
            r#"
            SELECT
                t.id, t.driver_id,
                p.full_name AS driver_name,
                t.vehicle_id, t.entered_by,
                t.trip_date,
                t.cash_aed, t.uber_cash_aed, t.bolt_cash_aed, t.card_aed,
                t.source AS "source: TripSource",
                t.notes, t.is_deleted, t.created_at
            FROM trips t
            JOIN drivers d ON d.id = t.driver_id
            JOIN profiles p ON p.id = d.profile_id
            WHERE t.is_deleted = false
              AND t.trip_date BETWEEN $2 AND $3
              AND ($1::uuid IS NULL OR t.driver_id = $1)
            ORDER BY t.trip_date DESC, t.created_at DESC
            "#,
            driver_id as Option<Uuid>,
            from,
            to
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Trip, AppError> {
        let row = sqlx::query_as!(
            Trip,
            r#"
            SELECT
                t.id, t.driver_id,
                p.full_name AS driver_name,
                t.vehicle_id, t.entered_by,
                t.trip_date,
                t.cash_aed, t.uber_cash_aed, t.bolt_cash_aed, t.card_aed,
                t.source AS "source: TripSource",
                t.notes, t.is_deleted, t.created_at
            FROM trips t
            JOIN drivers d ON d.id = t.driver_id
            JOIN profiles p ON p.id = d.profile_id
            WHERE t.id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Trip {id}")))?;

        Ok(row)
    }

    async fn create(&self, payload: CreateTrip) -> Result<Trip, AppError> {
        let row = sqlx::query_as!(
            Trip,
            r#"
            WITH ins AS (
                INSERT INTO trips (driver_id, vehicle_id, entered_by, trip_date, cash_aed, uber_cash_aed, bolt_cash_aed, card_aed, source, notes)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *
            )
            SELECT
                ins.id, ins.driver_id,
                p.full_name AS driver_name,
                ins.vehicle_id, ins.entered_by,
                ins.trip_date,
                ins.cash_aed, ins.uber_cash_aed, ins.bolt_cash_aed, ins.card_aed,
                ins.source AS "source: TripSource",
                ins.notes, ins.is_deleted, ins.created_at
            FROM ins
            JOIN drivers d ON d.id = ins.driver_id
            JOIN profiles p ON p.id = d.profile_id
            "#,
            payload.driver_id,
            payload.vehicle_id,
            payload.entered_by,
            payload.trip_date,
            payload.cash_aed,
            payload.uber_cash_aed,
            payload.bolt_cash_aed,
            payload.card_aed,
            payload.source as TripSource,
            payload.notes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    async fn update(&self, id: Uuid, payload: CreateTrip) -> Result<Trip, AppError> {
        let row = sqlx::query_as!(
            Trip,
            r#"
            WITH upd AS (
                UPDATE trips
                SET driver_id      = $2,
                    trip_date      = $3,
                    cash_aed       = $4,
                    uber_cash_aed  = $5,
                    bolt_cash_aed  = $6,
                    card_aed       = $7,
                    notes          = $8
                WHERE id = $1 AND is_deleted = false
                RETURNING *
            )
            SELECT
                upd.id, upd.driver_id,
                p.full_name AS driver_name,
                upd.vehicle_id, upd.entered_by,
                upd.trip_date,
                upd.cash_aed, upd.uber_cash_aed, upd.bolt_cash_aed, upd.card_aed,
                upd.source AS "source: TripSource",
                upd.notes, upd.is_deleted, upd.created_at
            FROM upd
            JOIN drivers d ON d.id = upd.driver_id
            JOIN profiles p ON p.id = d.profile_id
            "#,
            id,
            payload.driver_id,
            payload.trip_date,
            payload.cash_aed,
            payload.uber_cash_aed,
            payload.bolt_cash_aed,
            payload.card_aed,
            payload.notes
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Trip {id}")))?;

        Ok(row)
    }

    async fn soft_delete(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE trips SET is_deleted = true WHERE id = $1 AND is_deleted = false",
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Trip {id}")));
        }
        Ok(())
    }

    async fn daily_total(&self, driver_id: Uuid, date: NaiveDate) -> Result<Decimal, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(cash_aed + uber_cash_aed + bolt_cash_aed + card_aed), 0) AS "total!: Decimal"
            FROM trips
            WHERE driver_id = $1 AND trip_date = $2 AND is_deleted = false
            "#,
            driver_id,
            date
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.total)
    }

    async fn daily_total_excluding(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
        exclude_id: Uuid,
    ) -> Result<Decimal, AppError> {
        let row = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(cash_aed + uber_cash_aed + bolt_cash_aed + card_aed), 0) AS "total!: Decimal"
            FROM trips
            WHERE driver_id = $1 AND trip_date = $2 AND is_deleted = false AND id != $3
            "#,
            driver_id,
            date,
            exclude_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.total)
    }

    async fn bulk_insert(&self, rows: Vec<CreateTrip>) -> Result<Vec<Trip>, AppError> {
        let mut inserted = Vec::with_capacity(rows.len());
        for payload in rows {
            let trip = self.create(payload).await?;
            inserted.push(trip);
        }
        Ok(inserted)
    }

    async fn get_trip_cap(&self) -> Result<Decimal, AppError> {
        let row = sqlx::query!(
            "SELECT value FROM settings WHERE key = 'trip_cap_aed'"
        )
        .fetch_optional(&self.pool)
        .await?;

        let cap: Decimal = row
            .and_then(|r| r.value.parse().ok())
            .unwrap_or(Decimal::from(2000));

        Ok(cap)
    }

    async fn driver_self_entry_enabled(&self, driver_id: Uuid) -> Result<bool, AppError> {
        let row = sqlx::query!(
            "SELECT self_entry_enabled FROM drivers WHERE id = $1",
            driver_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Driver {driver_id}")))?;

        Ok(row.self_entry_enabled)
    }

    async fn find_driver_id_by_profile(&self, profile_id: Uuid) -> Result<Option<Uuid>, AppError> {
        let row = sqlx::query!(
            "SELECT id FROM drivers WHERE profile_id = $1",
            profile_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.id))
    }

    async fn monthly_earnings(&self, driver_id: Uuid, month: NaiveDate) -> Result<MonthlyEarnings, AppError> {
        use chrono::Datelike;
        // Compute first day of next month
        let (next_year, next_month) = if month.month() == 12 {
            (month.year() + 1, 1u32)
        } else {
            (month.year(), month.month() + 1)
        };
        let next_month_start = NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .ok_or_else(|| AppError::Internal("Date arithmetic overflow".into()))?;

        #[derive(sqlx::FromRow)]
        struct EarningsRow {
            cash_aed: Decimal,
            uber_cash_aed: Decimal,
            bolt_cash_aed: Decimal,
            card_aed: Decimal,
            total_aed: Decimal,
            trip_count: i64,
        }

        let row = sqlx::query_as::<_, EarningsRow>(
            r#"
            SELECT
                COALESCE(SUM(cash_aed), 0)                                               AS cash_aed,
                COALESCE(SUM(uber_cash_aed), 0)                                          AS uber_cash_aed,
                COALESCE(SUM(bolt_cash_aed), 0)                                          AS bolt_cash_aed,
                COALESCE(SUM(card_aed), 0)                                               AS card_aed,
                COALESCE(SUM(cash_aed + uber_cash_aed + bolt_cash_aed + card_aed), 0)    AS total_aed,
                COUNT(*)                                                                  AS trip_count
            FROM trips
            WHERE driver_id = $1
              AND trip_date >= $2
              AND trip_date < $3
              AND is_deleted = false
            "#,
        )
        .bind(driver_id)
        .bind(month)
        .bind(next_month_start)
        .fetch_one(&self.pool)
        .await?;

        Ok(MonthlyEarnings {
            cash_aed: row.cash_aed,
            uber_cash_aed: row.uber_cash_aed,
            bolt_cash_aed: row.bolt_cash_aed,
            card_aed: row.card_aed,
            total_aed: row.total_aed,
            trip_count: row.trip_count,
        })
    }

    async fn find_by_driver_and_date(&self, driver_id: Uuid, date: NaiveDate) -> Result<Vec<Trip>, AppError> {
        let rows = sqlx::query_as!(
            Trip,
            r#"
            SELECT
                t.id, t.driver_id,
                p.full_name AS driver_name,
                t.vehicle_id, t.entered_by,
                t.trip_date,
                t.cash_aed, t.uber_cash_aed, t.bolt_cash_aed, t.card_aed,
                t.source AS "source: TripSource",
                t.notes, t.is_deleted, t.created_at
            FROM trips t
            JOIN drivers d ON d.id = t.driver_id
            JOIN profiles p ON p.id = d.profile_id
            WHERE t.driver_id = $1
              AND t.trip_date = $2
              AND t.is_deleted = false
            ORDER BY t.created_at ASC
            "#,
            driver_id,
            date
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
