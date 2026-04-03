use async_trait::async_trait;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::owner::domain::{entity::Owner, repository::OwnerRepository};

pub struct PgOwnerRepository {
    pool: sqlx::PgPool,
}

impl PgOwnerRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OwnerRepository for PgOwnerRepository {
    async fn list(&self) -> Result<Vec<Owner>, AppError> {
        let rows = sqlx::query_as!(
            Owner,
            r#"SELECT o.id, o.profile_id, p.full_name, p.email,
                      p.phone, o.company_name, o.notes, o.is_active, o.created_at
               FROM owners o
               JOIN profiles p ON p.id = o.profile_id
               ORDER BY p.full_name"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Owner>, AppError> {
        let row = sqlx::query_as!(
            Owner,
            r#"SELECT o.id, o.profile_id, p.full_name, p.email,
                      p.phone, o.company_name, o.notes, o.is_active, o.created_at
               FROM owners o
               JOIN profiles p ON p.id = o.profile_id
               WHERE o.id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn find_by_profile_id(&self, profile_id: Uuid) -> Result<Option<Owner>, AppError> {
        let row = sqlx::query_as!(
            Owner,
            r#"SELECT o.id, o.profile_id, p.full_name, p.email,
                      p.phone, o.company_name, o.notes, o.is_active, o.created_at
               FROM owners o
               JOIN profiles p ON p.id = o.profile_id
               WHERE o.profile_id = $1"#,
            profile_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create(&self, profile_id: Uuid, company_name: Option<&str>, notes: Option<&str>) -> Result<Owner, AppError> {
        let row = sqlx::query_as!(
            Owner,
            r#"WITH ins AS (
                INSERT INTO owners (profile_id, company_name, notes) VALUES ($1, $2, $3)
                RETURNING *
            )
            SELECT ins.id, ins.profile_id, p.full_name, p.email,
                   p.phone, ins.company_name, ins.notes, ins.is_active, ins.created_at
            FROM ins
            JOIN profiles p ON p.id = ins.profile_id"#,
            profile_id, company_name, notes
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn update(&self, id: Uuid, company_name: Option<&str>, notes: Option<&str>) -> Result<Owner, AppError> {
        sqlx::query!(
            "UPDATE owners SET company_name = $2, notes = $3, updated_at = NOW() WHERE id = $1",
            id, company_name, notes
        )
        .execute(&self.pool)
        .await?;
        self.find_by_id(id).await?.ok_or_else(|| AppError::NotFound("Owner not found".into()))
    }
}
