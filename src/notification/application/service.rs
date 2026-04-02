use std::sync::Arc;

use crate::common::error::AppError;
use crate::notification::infrastructure::ResendClient;

pub struct NotificationService {
    resend: Arc<ResendClient>,
}

impl NotificationService {
    pub fn new(resend: Arc<ResendClient>) -> Self {
        Self { resend }
    }

    pub async fn send_invite_email(
        &self,
        to_email: &str,
        full_name: &str,
        invite_url: &str,
    ) -> Result<(), AppError> {
        let subject = "You've been invited to Voiture Voyages FMS";
        let html = format!(
            r#"<p>Hello {full_name},</p>
<p>You have been invited to join the Fleet Management System.</p>
<p><a href="{invite_url}" style="background:#6366f1;color:#fff;padding:10px 20px;border-radius:6px;text-decoration:none;display:inline-block">Accept Invitation</a></p>
<p>This link expires in 24 hours. If you didn't expect this email, you can safely ignore it.</p>
<p>— Voiture Voyages Operations</p>"#
        );
        self.resend.send(to_email, subject, &html).await
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_url: &str,
    ) -> Result<(), AppError> {
        let subject = "Reset your FMS password";
        let html = format!(
            r#"<p>Hello,</p>
<p>We received a request to reset your password.</p>
<p><a href="{reset_url}" style="background:#6366f1;color:#fff;padding:10px 20px;border-radius:6px;text-decoration:none;display:inline-block">Reset Password</a></p>
<p>This link expires in 1 hour. If you didn't request this, ignore this email.</p>
<p>— Voiture Voyages Operations</p>"#
        );
        self.resend.send(to_email, subject, &html).await
    }
}
