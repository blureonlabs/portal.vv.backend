use std::sync::Arc;
use chrono::Utc;
use rand::RngCore;
use sha2::{Sha256, Digest};
use uuid::Uuid;

use crate::audit::application::service::AuditService;
use crate::common::{error::AppError, types::Role};
use crate::config::AppConfig;
use crate::auth::domain::{
    entity::InviteStatus,
    repository::AuthRepository,
};
use crate::auth::infrastructure::SupabaseAdminClient;
use crate::notification::application::service::NotificationService;

pub struct AuthService {
    pub repo: Arc<dyn AuthRepository>,
    pub supabase: Arc<SupabaseAdminClient>,
    pub config: Arc<AppConfig>,
    pub notification: Arc<NotificationService>,
    pub audit: Arc<AuditService>,
}

impl AuthService {
    pub fn new(
        repo: Arc<dyn AuthRepository>,
        supabase: Arc<SupabaseAdminClient>,
        config: Arc<AppConfig>,
        notification: Arc<NotificationService>,
        audit: Arc<AuditService>,
    ) -> Self {
        Self { repo, supabase, config, notification, audit }
    }

    fn generate_token() -> (String, String) {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let raw = hex::encode(bytes);
        let hash = hex::encode(Sha256::digest(raw.as_bytes()));
        (raw, hash)
    }

    pub fn hash_token(raw: &str) -> String {
        hex::encode(Sha256::digest(raw.as_bytes()))
    }

    pub async fn invite_user(
        &self,
        invited_by: Uuid,
        actor_role: &Role,
        email: String,
        full_name: String,
        role: Role,
    ) -> Result<Uuid, AppError> {
        if self.repo.find_profile_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict(format!("{} is already registered", email)));
        }

        let (raw_token, token_hash) = Self::generate_token();
        let invite = self.repo.create_invite(&email, role, &token_hash, invited_by).await?;

        let invite_url = format!("{}/accept-invite?token={}", self.config.frontend_url, raw_token);
        self.notification.send_invite_email(&email, &full_name, &invite_url).await?;

        self.audit.log(
            invited_by,
            actor_role,
            "invite",
            Some(invite.id),
            "created",
            Some(serde_json::json!({ "email": email })),
        ).await?;

        Ok(invite.id)
    }

    pub async fn accept_invite(
        &self,
        raw_token: String,
        full_name: String,
        password: String,
    ) -> Result<(), AppError> {
        let hash = Self::hash_token(&raw_token);
        let invite = self.repo.find_invite_by_token_hash(&hash).await?
            .ok_or_else(|| AppError::BadRequest("Invalid or expired invite link".into()))?;

        if invite.status != InviteStatus::Pending {
            return Err(AppError::BadRequest("This invite has already been used or revoked".into()));
        }
        if invite.expires_at < Utc::now() {
            self.repo.update_invite_status(invite.id, InviteStatus::Expired).await?;
            return Err(AppError::BadRequest("This invite has expired. Contact your administrator.".into()));
        }

        let user_id = self.supabase.create_user(&invite.email, &password).await?;

        // Compensate: if DB operations fail, delete the Supabase user we just created
        if let Err(e) = self.repo.insert_profile(user_id, invite.role.clone(), &full_name, &invite.email, Some(invite.invited_by)).await {
            let _ = self.supabase.delete_user(user_id).await;
            return Err(e);
        }
        if let Err(e) = self.repo.update_invite_status(invite.id, InviteStatus::Accepted).await {
            let _ = self.supabase.delete_user(user_id).await;
            return Err(e);
        }

        self.audit.log(
            user_id,
            &invite.role,
            "invite",
            Some(invite.id),
            "accepted",
            None,
        ).await?;

        Ok(())
    }

    pub async fn revoke_invite(&self, actor_id: Uuid, actor_role: &Role, invite_id: Uuid) -> Result<(), AppError> {
        let invite = self.repo.find_invite_by_id(invite_id).await?
            .ok_or_else(|| AppError::NotFound("Invite not found".into()))?;

        if invite.status != InviteStatus::Pending {
            return Err(AppError::BadRequest("Only pending invites can be revoked".into()));
        }

        self.repo.update_invite_status(invite_id, InviteStatus::Revoked).await?;

        self.audit.log(
            actor_id,
            actor_role,
            "invite",
            Some(invite_id),
            "revoked",
            None,
        ).await?;

        Ok(())
    }

    pub async fn resend_invite(&self, actor_id: Uuid, actor_role: &Role, invite_id: Uuid) -> Result<(), AppError> {
        let invite = self.repo.find_invite_by_id(invite_id).await?
            .ok_or_else(|| AppError::NotFound("Invite not found".into()))?;

        if invite.status == InviteStatus::Accepted {
            return Err(AppError::BadRequest("Invite already accepted".into()));
        }

        let (raw_token, token_hash) = Self::generate_token();
        self.repo.update_invite_token(invite_id, &token_hash).await?;

        let invite_url = format!("{}/accept-invite?token={}", self.config.frontend_url, raw_token);
        self.notification.send_invite_email(&invite.email, "Team Member", &invite_url).await?;

        self.audit.log(
            actor_id,
            actor_role,
            "invite",
            Some(invite_id),
            "resent",
            None,
        ).await?;

        Ok(())
    }

    pub async fn send_password_reset_link(
        &self,
        actor_id: Uuid,
        actor_role: &Role,
        target_user_id: Uuid,
    ) -> Result<(), AppError> {
        let profile = self.repo
            .find_profile_by_id(target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        let reset_url = self.supabase
            .generate_recovery_link(target_user_id, &self.config.frontend_url)
            .await?;

        self.notification
            .send_password_reset_email(&profile.email, &reset_url)
            .await?;

        self.audit.log(actor_id, actor_role, "auth", Some(target_user_id), "password.reset_link_sent",
            Some(serde_json::json!({ "target_email": profile.email }))).await?;

        Ok(())
    }

    pub async fn forgot_password(&self, email: &str) -> Result<(), AppError> {
        let profile = self.repo.find_profile_by_email(email).await?;
        if profile.is_none() {
            // Return Ok to avoid user enumeration
            return Ok(());
        }
        let profile = profile.unwrap();

        // Use Supabase admin API to generate a recovery link
        let reset_url = self.supabase
            .generate_recovery_link(profile.id, &self.config.frontend_url)
            .await?;

        self.notification.send_password_reset_email(email, &reset_url).await?;
        Ok(())
    }
}

/// Bootstrap first Super Admin — called from CLI
pub async fn seed_admin(pool: &sqlx::PgPool, email: &str, config: &AppConfig) -> anyhow::Result<()> {
    if config.seed_key.is_empty() {
        anyhow::bail!("SEED_KEY is not set.");
    }

    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM profiles WHERE role = 'super_admin'"
    )
    .fetch_one(pool)
    .await?;

    if row.0 > 0 {
        anyhow::bail!("A Super Admin already exists. Aborting.");
    }

    let supabase = SupabaseAdminClient::new(config);

    println!("Enter password for {}: ", email);
    let mut password = String::new();
    std::io::stdin().read_line(&mut password)?;
    let password = password.trim();

    let user_id = supabase.create_user(email, password).await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    sqlx::query!(
        "INSERT INTO profiles (id, role, full_name, email) VALUES ($1, 'super_admin', 'Super Admin', $2)",
        user_id,
        email
    )
    .execute(pool)
    .await?;

    println!("Super Admin created: {}", email);
    println!("Remove SEED_KEY from environment now.");
    Ok(())
}
