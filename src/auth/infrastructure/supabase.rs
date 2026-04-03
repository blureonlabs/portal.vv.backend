use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::common::error::AppError;
use crate::config::AppConfig;

#[derive(Deserialize)]
struct GenerateLinkResponse {
    action_link: String,
}

pub struct SupabaseAdminClient {
    http: reqwest::Client,
    base_url: String,
    service_role_key: String,
}

#[derive(Serialize)]
struct CreateUserPayload<'a> {
    email: &'a str,
    password: &'a str,
    email_confirm: bool,
}

#[derive(Deserialize)]
struct SupabaseUser {
    id: Uuid,
}

#[derive(Serialize)]
struct UpdateUserPayload<'a> {
    password: &'a str,
}

impl SupabaseAdminClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: format!("{}/auth/v1", config.supabase_url),
            service_role_key: config.supabase_service_role_key.clone(),
        }
    }

    pub async fn create_user(&self, email: &str, password: &str) -> Result<Uuid, AppError> {
        let res = self.http
            .post(format!("{}/admin/users", self.base_url))
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .json(&CreateUserPayload { email, password, email_confirm: true })
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Supabase create_user failed: {}", body)));
        }

        let user: SupabaseUser = res.json().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(user.id)
    }

    #[allow(dead_code)]
    pub async fn disable_user(&self, user_id: Uuid) -> Result<(), AppError> {
        #[derive(Serialize)]
        struct Payload { ban_duration: &'static str }

        let res = self.http
            .put(format!("{}/admin/users/{}", self.base_url, user_id))
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .json(&Payload { ban_duration: "876600h" }) // ~100 years
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Supabase disable_user failed: {}", body)));
        }
        Ok(())
    }

    pub async fn generate_recovery_link(&self, user_id: Uuid, frontend_url: &str) -> Result<String, AppError> {
        #[derive(Serialize)]
        struct Payload<'a> {
            #[serde(rename = "type")]
            kind: &'static str,
            redirect_to: &'a str,
        }

        let res = self.http
            .post(format!("{}/admin/users/{}/generate-link", self.base_url, user_id))
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .json(&Payload { kind: "recovery", redirect_to: &format!("{}/reset-password", frontend_url) })
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Supabase generate_link failed: {}", body)));
        }

        let data: GenerateLinkResponse = res.json().await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(data.action_link)
    }

    #[allow(dead_code)]
    pub async fn enable_user(&self, user_id: Uuid) -> Result<(), AppError> {
        #[derive(Serialize)]
        struct Payload { ban_duration: &'static str }

        let res = self.http
            .put(format!("{}/admin/users/{}", self.base_url, user_id))
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .json(&Payload { ban_duration: "none" })
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Supabase enable_user failed: {}", body)));
        }
        Ok(())
    }

    pub async fn update_user_password(&self, user_id: Uuid, new_password: &str) -> Result<(), AppError> {
        let res = self.http
            .put(format!("{}/admin/users/{}", self.base_url, user_id))
            .header("apikey", &self.service_role_key)
            .header("Authorization", format!("Bearer {}", self.service_role_key))
            .json(&UpdateUserPayload { password: new_password })
            .send()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Supabase update_user_password failed: {}", body)));
        }
        Ok(())
    }
}
