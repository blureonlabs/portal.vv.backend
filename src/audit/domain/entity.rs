use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub actor_role: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub action: String,
    pub metadata_json: Option<Value>,
    pub created_at: DateTime<Utc>,
}
