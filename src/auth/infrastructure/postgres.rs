use async_trait::async_trait;
use chrono::{Duration, Utc};
use uuid::Uuid;
use crate::common::{error::AppError, types::Role};
use crate::auth::domain::{
    entity::{Invite, InviteStatus, Profile},
    repository::AuthRepository,
};

pub struct PgAuthRepository {
    pool: sqlx::PgPool,
}

impl PgAuthRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthRepository for PgAuthRepository {
    async fn find_profile_by_id(&self, id: Uuid) -> Result<Option<Profile>, AppError> {
        let profile = sqlx::query_as!(
            Profile,
            r#"SELECT id, role as "role: Role", full_name, email, is_active, avatar_url, invited_by, created_at
               FROM profiles WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(profile)
    }

    async fn find_profile_by_email(&self, email: &str) -> Result<Option<Profile>, AppError> {
        let profile = sqlx::query_as!(
            Profile,
            r#"SELECT id, role as "role: Role", full_name, email, is_active, avatar_url, invited_by, created_at
               FROM profiles WHERE email = $1"#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(profile)
    }

    async fn list_profiles(&self) -> Result<Vec<Profile>, AppError> {
        let profiles = sqlx::query_as!(
            Profile,
            r#"SELECT id, role as "role: Role", full_name, email, is_active, avatar_url, invited_by, created_at
               FROM profiles ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(profiles)
    }

    async fn insert_profile(&self, id: Uuid, role: Role, full_name: &str, email: &str, invited_by: Option<Uuid>) -> Result<Profile, AppError> {
        let profile = sqlx::query_as!(
            Profile,
            r#"INSERT INTO profiles (id, role, full_name, email, invited_by)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, role as "role: Role", full_name, email, is_active, avatar_url, invited_by, created_at"#,
            id,
            role as Role,
            full_name,
            email,
            invited_by,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(profile)
    }

    async fn create_profile(&self, id: Uuid, full_name: &str, email: &str, role: &Role, phone: Option<&str>) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO profiles (id, role, full_name, email, phone) VALUES ($1, $2, $3, $4, $5)",
            id, role as &Role, full_name, email, phone
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn create_invite(&self, email: &str, role: Role, token_hash: &str, invited_by: Uuid) -> Result<Invite, AppError> {
        let expires_at = Utc::now() + Duration::hours(24);
        let invite = sqlx::query_as!(
            Invite,
            r#"INSERT INTO invites (email, role, token_hash, invited_by, expires_at)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, email, role as "role: Role", token_hash, invited_by,
                         status as "status: InviteStatus", expires_at, created_at"#,
            email,
            role as Role,
            token_hash,
            invited_by,
            expires_at,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(invite)
    }

    async fn find_invite_by_token_hash(&self, token_hash: &str) -> Result<Option<Invite>, AppError> {
        let invite = sqlx::query_as!(
            Invite,
            r#"SELECT id, email, role as "role: Role", token_hash, invited_by,
                      status as "status: InviteStatus", expires_at, created_at
               FROM invites WHERE token_hash = $1"#,
            token_hash
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(invite)
    }

    async fn find_invite_by_id(&self, id: Uuid) -> Result<Option<Invite>, AppError> {
        let invite = sqlx::query_as!(
            Invite,
            r#"SELECT id, email, role as "role: Role", token_hash, invited_by,
                      status as "status: InviteStatus", expires_at, created_at
               FROM invites WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(invite)
    }

    async fn list_invites(&self) -> Result<Vec<Invite>, AppError> {
        let invites = sqlx::query_as!(
            Invite,
            r#"SELECT id, email, role as "role: Role", token_hash, invited_by,
                      status as "status: InviteStatus", expires_at, created_at
               FROM invites ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(invites)
    }

    async fn update_invite_status(&self, id: Uuid, status: InviteStatus) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE invites SET status = $1, updated_at = NOW() WHERE id = $2",
            status as InviteStatus,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_invite_token(&self, id: Uuid, token_hash: &str) -> Result<(), AppError> {
        let expires_at = Utc::now() + chrono::Duration::hours(24);
        sqlx::query!(
            "UPDATE invites SET token_hash = $1, expires_at = $2, status = 'pending', updated_at = NOW() WHERE id = $3",
            token_hash,
            expires_at,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_avatar(&self, id: Uuid, avatar_url: &str) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE profiles SET avatar_url = $2 WHERE id = $1",
            id,
            avatar_url,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
