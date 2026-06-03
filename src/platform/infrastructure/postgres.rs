use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::common::error::AppError;
use crate::platform::domain::{
    entity::{Platform, CreatePlatform, UpdatePlatform},
    repository::PlatformRepository,
};

pub struct PgPlatformRepository {
    pool: PgPool,
}

impl PgPlatformRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlatformRepository for PgPlatformRepository {
    async fn list_active(&self) -> Result<Vec<Platform>, AppError> {
        let rows = sqlx::query_as::<_, Platform>(
            "SELECT id, name, code, is_active, sort_order, created_at \
             FROM platforms WHERE is_active = true ORDER BY sort_order, name"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn create(&self, payload: CreatePlatform) -> Result<Platform, AppError> {
        let row = sqlx::query_as::<_, Platform>(
            "INSERT INTO platforms (name, code) VALUES ($1, $2) \
             RETURNING id, name, code, is_active, sort_order, created_at"
        )
        .bind(&payload.name)
        .bind(&payload.code)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn update(&self, id: Uuid, payload: UpdatePlatform) -> Result<Platform, AppError> {
        let row = sqlx::query_as::<_, Platform>(
            "UPDATE platforms SET name = $1, is_active = $2, sort_order = $3 WHERE id = $4 \
             RETURNING id, name, code, is_active, sort_order, created_at"
        )
        .bind(&payload.name)
        .bind(payload.is_active)
        .bind(payload.sort_order)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Platform not found".into()))?;
        Ok(row)
    }

    async fn deactivate(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE platforms SET is_active = false WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Platform not found".into()));
        }
        Ok(())
    }
}
