use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Platform {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

pub struct CreatePlatform {
    pub name: String,
    pub code: String,
}

pub struct UpdatePlatform {
    pub name: String,
    pub is_active: bool,
    pub sort_order: i32,
}
