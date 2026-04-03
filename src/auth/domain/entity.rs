use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::common::types::Role;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub id: Uuid,
    pub role: Role,
    pub full_name: String,
    pub email: String,
    pub is_active: bool,
    pub avatar_url: Option<String>,
    pub invited_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "invite_status", rename_all = "snake_case")]
pub enum InviteStatus {
    Pending,
    Accepted,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invite {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
    pub token_hash: String,
    pub invited_by: Uuid,
    pub status: InviteStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
