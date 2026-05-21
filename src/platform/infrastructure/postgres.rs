use async_trait::async_trait;
use sqlx::PgPool;
use crate::common::error::AppError;
use crate::platform::domain::{entity::Platform, repository::PlatformRepository};

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
            "SELECT id, name, code, is_active, created_at FROM platforms WHERE is_active = true ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
