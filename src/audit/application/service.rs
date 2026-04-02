use serde_json::Value;
use uuid::Uuid;

use crate::common::{error::AppError, types::Role};

pub struct AuditService {
    pool: sqlx::PgPool,
}

impl AuditService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn log(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        entity_type: &str,
        entity_id: Option<Uuid>,
        action: &str,
        metadata: Option<Value>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"INSERT INTO audit_log (actor_id, actor_role, entity_type, entity_id, action, metadata_json)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
            actor_id,
            actor_role as &Role,
            entity_type,
            entity_id,
            action,
            metadata
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
