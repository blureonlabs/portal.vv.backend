use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::vehicle::domain::{
    entity::{Vehicle, VehicleAssignment, VehicleServiceRecord, VehicleStatus},
    repository::VehicleRepository,
};

pub struct PgVehicleRepository {
    pool: sqlx::PgPool,
}

impl PgVehicleRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleRepository for PgVehicleRepository {
    async fn list(&self) -> Result<Vec<Vehicle>, AppError> {
        let rows = sqlx::query_as!(
            Vehicle,
            r#"SELECT v.id, v.plate_number, v.make, v.model, v.year, v.color,
                      v.registration_date, v.registration_expiry, v.insurance_expiry,
                      v.status as "status: VehicleStatus", v.is_active, v.created_at,
                      va.driver_id as assigned_driver_id,
                      p.full_name as assigned_driver_name
               FROM vehicles v
               LEFT JOIN vehicle_assignments va ON va.vehicle_id = v.id AND va.unassigned_at IS NULL
               LEFT JOIN drivers d ON d.id = va.driver_id
               LEFT JOIN profiles p ON p.id = d.profile_id
               ORDER BY v.plate_number"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Vehicle>, AppError> {
        let row = sqlx::query_as!(
            Vehicle,
            r#"SELECT v.id, v.plate_number, v.make, v.model, v.year, v.color,
                      v.registration_date, v.registration_expiry, v.insurance_expiry,
                      v.status as "status: VehicleStatus", v.is_active, v.created_at,
                      va.driver_id as assigned_driver_id,
                      p.full_name as assigned_driver_name
               FROM vehicles v
               LEFT JOIN vehicle_assignments va ON va.vehicle_id = v.id AND va.unassigned_at IS NULL
               LEFT JOIN drivers d ON d.id = va.driver_id
               LEFT JOIN profiles p ON p.id = d.profile_id
               WHERE v.id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create(
        &self,
        plate_number: &str,
        make: &str,
        model: &str,
        year: i32,
        color: Option<&str>,
        registration_date: Option<NaiveDate>,
        registration_expiry: Option<NaiveDate>,
        insurance_expiry: Option<NaiveDate>,
    ) -> Result<Vehicle, AppError> {
        let row = sqlx::query_as!(
            Vehicle,
            r#"INSERT INTO vehicles (plate_number, make, model, year, color, registration_date, registration_expiry, insurance_expiry)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING id, plate_number, make, model, year, color,
                         registration_date, registration_expiry, insurance_expiry,
                         status as "status: VehicleStatus", is_active, created_at,
                         NULL::uuid as assigned_driver_id, NULL::text as assigned_driver_name"#,
            plate_number, make, model, year, color,
            registration_date, registration_expiry, insurance_expiry,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn update(
        &self,
        id: Uuid,
        plate_number: &str,
        make: &str,
        model: &str,
        year: i32,
        color: Option<&str>,
        registration_date: Option<NaiveDate>,
        registration_expiry: Option<NaiveDate>,
        insurance_expiry: Option<NaiveDate>,
    ) -> Result<Vehicle, AppError> {
        sqlx::query!(
            r#"UPDATE vehicles SET plate_number=$2, make=$3, model=$4, year=$5, color=$6,
               registration_date=$7, registration_expiry=$8, insurance_expiry=$9, updated_at=NOW()
               WHERE id=$1"#,
            id, plate_number, make, model, year, color,
            registration_date, registration_expiry, insurance_expiry,
        )
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or_else(|| AppError::NotFound("Vehicle not found".into()))
    }

    async fn assign(&self, vehicle_id: Uuid, driver_id: Uuid, assigned_by: Uuid) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "INSERT INTO vehicle_assignments (vehicle_id, driver_id, assigned_by) VALUES ($1, $2, $3)",
            vehicle_id, driver_id, assigned_by
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE vehicles SET status = 'assigned', updated_at = NOW() WHERE id = $1",
            vehicle_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn unassign(&self, vehicle_id: Uuid) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE vehicle_assignments SET unassigned_at = NOW() WHERE vehicle_id = $1 AND unassigned_at IS NULL",
            vehicle_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE vehicles SET status = 'available', updated_at = NOW() WHERE id = $1",
            vehicle_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn driver_has_active_assignment(&self, driver_id: Uuid) -> Result<bool, AppError> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM vehicle_assignments WHERE driver_id = $1 AND unassigned_at IS NULL)"
        )
        .bind(driver_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    async fn list_assignments(&self, vehicle_id: Uuid) -> Result<Vec<VehicleAssignment>, AppError> {
        let rows = sqlx::query_as!(
            VehicleAssignment,
            r#"SELECT va.id, va.vehicle_id, va.driver_id, p.full_name as driver_name,
                      va.assigned_at, va.unassigned_at, va.assigned_by
               FROM vehicle_assignments va
               JOIN drivers d ON d.id = va.driver_id
               JOIN profiles p ON p.id = d.profile_id
               WHERE va.vehicle_id = $1
               ORDER BY va.assigned_at DESC"#,
            vehicle_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn add_service_record(
        &self,
        vehicle_id: Uuid,
        service_date: NaiveDate,
        service_type: &str,
        description: Option<&str>,
        cost: Decimal,
        next_due: Option<NaiveDate>,
        logged_by: Uuid,
    ) -> Result<VehicleServiceRecord, AppError> {
        let row = sqlx::query_as!(
            VehicleServiceRecord,
            r#"INSERT INTO vehicle_service (vehicle_id, service_date, type, description, cost, next_due, logged_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, vehicle_id, service_date, type as service_type,
                         description, cost, next_due, logged_by, created_at"#,
            vehicle_id, service_date, service_type, description, cost, next_due, logged_by,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn list_service_records(&self, vehicle_id: Uuid) -> Result<Vec<VehicleServiceRecord>, AppError> {
        let rows = sqlx::query_as!(
            VehicleServiceRecord,
            r#"SELECT id, vehicle_id, service_date, type as service_type,
                      description, cost, next_due, logged_by, created_at
               FROM vehicle_service WHERE vehicle_id = $1 ORDER BY service_date DESC"#,
            vehicle_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
