use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Platform {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
