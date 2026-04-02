use async_trait::async_trait;
use uuid::Uuid;
use crate::common::{error::AppError, types::Role};
use super::entity::{Invite, InviteStatus, Profile};

#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn find_profile_by_id(&self, id: Uuid) -> Result<Option<Profile>, AppError>;
    async fn find_profile_by_email(&self, email: &str) -> Result<Option<Profile>, AppError>;
    async fn list_profiles(&self) -> Result<Vec<Profile>, AppError>;
    async fn insert_profile(&self, id: Uuid, role: Role, full_name: &str, email: &str, invited_by: Option<Uuid>) -> Result<Profile, AppError>;

    async fn create_invite(&self, email: &str, role: Role, token_hash: &str, invited_by: Uuid) -> Result<Invite, AppError>;
    async fn find_invite_by_token_hash(&self, token_hash: &str) -> Result<Option<Invite>, AppError>;
    async fn find_invite_by_id(&self, id: Uuid) -> Result<Option<Invite>, AppError>;
    async fn list_invites(&self) -> Result<Vec<Invite>, AppError>;
    async fn update_invite_status(&self, id: Uuid, status: InviteStatus) -> Result<(), AppError>;
    async fn update_invite_token(&self, id: Uuid, token_hash: &str) -> Result<(), AppError>;
}
