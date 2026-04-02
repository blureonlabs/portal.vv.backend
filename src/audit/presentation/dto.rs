use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::audit::domain::entity::AuditEntry;

#[derive(Debug, Deserialize)]
pub struct ListAuditQuery {
    pub entity_type: Option<String>,
    pub actor_id: Option<Uuid>,
    pub action: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize)]
pub struct AuditEntryResponse {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub actor_role: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub action: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditEntry> for AuditEntryResponse {
    fn from(e: AuditEntry) -> Self {
        Self {
            id: e.id,
            actor_id: e.actor_id,
            actor_role: e.actor_role,
            entity_type: e.entity_type,
            entity_id: e.entity_id,
            action: e.action,
            metadata: e.metadata_json,
            created_at: e.created_at,
        }
    }
}
