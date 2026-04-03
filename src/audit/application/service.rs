use serde_json::Value;
use uuid::Uuid;

use crate::common::{error::AppError, types::Role};
use crate::audit::domain::entity::AuditEntry;

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

    pub async fn list(
        &self,
        entity_type: Option<&str>,
        actor_id: Option<Uuid>,
        action: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditEntry>, AppError> {
        // Build dynamic query — no compile-time macro because conditions are runtime
        let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new(
            "SELECT id, actor_id, actor_role::text, entity_type, entity_id, action, metadata_json, created_at \
             FROM audit_log WHERE 1=1",
        );
        if let Some(et) = entity_type {
            qb.push(" AND entity_type = ").push_bind(et);
        }
        if let Some(aid) = actor_id {
            qb.push(" AND actor_id = ").push_bind(aid);
        }
        if let Some(act) = action {
            qb.push(" AND action = ").push_bind(act);
        }
        qb.push(" ORDER BY created_at DESC LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind(offset);

        let rows = qb
            .build_query_as::<AuditEntryRow>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| AuditEntry {
            id: r.id,
            actor_id: r.actor_id,
            actor_role: r.actor_role,
            entity_type: r.entity_type,
            entity_id: r.entity_id,
            action: r.action,
            metadata_json: r.metadata_json,
            created_at: r.created_at,
        }).collect())
    }
}

#[derive(sqlx::FromRow)]
struct AuditEntryRow {
    id: Uuid,
    actor_id: Uuid,
    actor_role: String,
    entity_type: String,
    entity_id: Option<Uuid>,
    action: String,
    metadata_json: Option<Value>,
    created_at: chrono::DateTime<chrono::Utc>,
}
