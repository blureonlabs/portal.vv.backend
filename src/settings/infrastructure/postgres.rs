use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::settings::domain::{entity::Setting, repository::SettingsRepository};

pub struct PgSettingsRepository {
    pool: PgPool,
}

impl PgSettingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsRepository for PgSettingsRepository {
    async fn list(&self) -> Result<Vec<Setting>, AppError> {
        let rows = sqlx::query_as!(
            Setting,
            "SELECT id, key, value, updated_by, updated_at FROM settings ORDER BY key"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn get(&self, key: &str) -> Result<Option<Setting>, AppError> {
        let row = sqlx::query_as!(
            Setting,
            "SELECT id, key, value, updated_by, updated_at FROM settings WHERE key = $1",
            key
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn upsert(&self, key: &str, value: &str, updated_by: Uuid) -> Result<Setting, AppError> {
        let row = sqlx::query_as!(
            Setting,
            r#"
            INSERT INTO settings (key, value, updated_by)
            VALUES ($1, $2, $3)
            ON CONFLICT (key) DO UPDATE
              SET value = EXCLUDED.value,
                  updated_by = EXCLUDED.updated_by,
                  updated_at = NOW()
            RETURNING id, key, value, updated_by, updated_at
            "#,
            key,
            value,
            updated_by
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }
}
