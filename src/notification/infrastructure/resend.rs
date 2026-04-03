use reqwest::Client;
use serde::Serialize;

use crate::common::error::AppError;
use crate::config::AppConfig;

pub struct ResendClient {
    http: Client,
    api_key: String,
    from_email: String,
}

#[derive(Serialize)]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    html: &'a str,
}

impl ResendClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            http: Client::new(),
            api_key: config.resend_api_key.clone(),
            from_email: config.resend_from_email.clone(),
        }
    }

    pub async fn send(&self, to: &str, subject: &str, html: &str) -> Result<(), AppError> {
        let res = self.http
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&SendEmailRequest {
                from: &self.from_email,
                to: vec![to],
                subject,
                html,
            })
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Resend request failed: {}", e)))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Resend API error: {}", body)));
        }
        Ok(())
    }

    /// Send a broadcast email with custom subject and HTML body.
    pub async fn send_broadcast_email(&self, to: &str, subject: &str, html_body: &str) -> Result<(), AppError> {
        let res = self.http
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&SendEmailRequest {
                from: &self.from_email,
                to: vec![to],
                subject,
                html: html_body,
            })
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Resend request failed: {}", e)))?;

        if !res.status().is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Resend API error: {}", body)));
        }
        Ok(())
    }
}
