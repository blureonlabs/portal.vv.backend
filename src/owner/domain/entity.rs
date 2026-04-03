use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Owner {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
