use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::domain::entity::{Invite, InviteStatus, Profile};
use crate::common::types::Role;

// ---------- Requests ----------

#[derive(Debug, Deserialize)]
pub struct InviteUserRequest {
    pub email: String,
    pub full_name: String,
    pub role: Role,
}

#[derive(Debug, Deserialize)]
pub struct AcceptInviteRequest {
    pub token: String,
    pub full_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAvatarRequest {
    pub avatar_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub password: String,
}

// ---------- Responses ----------

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
    pub status: InviteStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<Invite> for InviteResponse {
    fn from(i: Invite) -> Self {
        Self {
            id: i.id,
            email: i.email,
            role: i.role,
            status: i.status,
            expires_at: i.expires_at,
            created_at: i.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: Role,
    pub is_active: bool,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Profile> for UserResponse {
    fn from(p: Profile) -> Self {
        Self {
            id: p.id,
            email: p.email,
            full_name: p.full_name,
            role: p.role,
            is_active: p.is_active,
            avatar_url: p.avatar_url,
            created_at: p.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: Role,
    pub is_active: bool,
    pub avatar_url: Option<String>,
}

impl From<Profile> for MeResponse {
    fn from(p: Profile) -> Self {
        Self {
            id: p.id,
            email: p.email,
            full_name: p.full_name,
            role: p.role,
            is_active: p.is_active,
            avatar_url: p.avatar_url,
        }
    }
}
