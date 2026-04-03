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

    pub async fn send_broadcast_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), AppError> {
        let html = format!(
            r#"<div style="font-family:sans-serif;max-width:600px;margin:0 auto">
{html_body}
<hr style="border:none;border-top:1px solid #e5e7eb;margin:24px 0">
<p style="color:#6b7280;font-size:12px">Sent via Voiture Voyages Fleet Management</p>
</div>"#
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

    pub async fn send_advance_status_email(
        &self,
        to: &str,
        driver_name: &str,
        amount: &str,
        status: &str,
        reason: Option<&str>,
    ) -> Result<(), AppError> {
        let subject = format!("Advance Request {}", Self::capitalise(status));
        let status_color = match status {
            "approved" => "#16a34a",
            "rejected" => "#dc2626",
            "paid"     => "#161f3f",
            _          => "#6b7280",
        };
        let reason_block = reason
            .map(|r| format!(
                r#"<p style="margin:16px 0"><strong>Reason:</strong> {r}</p>"#
            ))
            .unwrap_or_default();
        let html = format!(
            r#"<div style="font-family:sans-serif;max-width:600px;margin:0 auto">
  <div style="background:#161f3f;padding:24px 32px;border-radius:8px 8px 0 0">
    <h1 style="color:#fff;margin:0;font-size:20px">Voiture Voyages Fleet Management</h1>
  </div>
  <div style="padding:32px;border:1px solid #e5e7eb;border-top:none;border-radius:0 0 8px 8px">
    <p style="margin:0 0 16px">Hello <strong>{driver_name}</strong>,</p>
    <p style="margin:0 0 16px">Your advance request for <strong>AED {amount}</strong> has been
      <span style="color:{status_color};font-weight:600">{status}</span>.</p>
    {reason_block}
    <p style="margin:24px 0 0;color:#6b7280;font-size:13px">If you have any questions, please contact the operations team.</p>
  </div>
  <p style="color:#9ca3af;font-size:11px;text-align:center;margin:16px 0">© Voiture Voyages — Fleet Management System</p>
</div>"#
        );
        self.resend.send(to, &subject, &html).await
    }

    pub async fn send_advance_request_notification(
        &self,
        to: &str,
        driver_name: &str,
        amount: &str,
    ) -> Result<(), AppError> {
        let subject = "New Advance Request — Action Required";
        let html = format!(
            r#"<div style="font-family:sans-serif;max-width:600px;margin:0 auto">
  <div style="background:#161f3f;padding:24px 32px;border-radius:8px 8px 0 0">
    <h1 style="color:#fff;margin:0;font-size:20px">Voiture Voyages Fleet Management</h1>
  </div>
  <div style="padding:32px;border:1px solid #e5e7eb;border-top:none;border-radius:0 0 8px 8px">
    <p style="margin:0 0 16px">A new advance request requires your attention.</p>
    <p style="margin:0 0 16px"><strong>Driver:</strong> {driver_name}<br>
       <strong>Amount:</strong> AED {amount}</p>
    <p style="margin:0 0 16px">Please log in to the Fleet Management System to review and action this request.</p>
    <p style="margin:24px 0 0;color:#6b7280;font-size:13px">This is an automated notification.</p>
  </div>
  <p style="color:#9ca3af;font-size:11px;text-align:center;margin:16px 0">© Voiture Voyages — Fleet Management System</p>
</div>"#
        );
        self.resend.send(to, subject, &html).await
    }

    pub async fn send_leave_status_email(
        &self,
        to: &str,
        driver_name: &str,
        leave_type: &str,
        dates: &str,
        status: &str,
        reason: Option<&str>,
    ) -> Result<(), AppError> {
        let subject = format!("Leave Request {}", Self::capitalise(status));
        let status_color = match status {
            "approved" => "#16a34a",
            "rejected" => "#dc2626",
            _          => "#6b7280",
        };
        let reason_block = reason
            .map(|r| format!(
                r#"<p style="margin:16px 0"><strong>Reason:</strong> {r}</p>"#
            ))
            .unwrap_or_default();
        let leave_label = Self::capitalise(leave_type);
        let html = format!(
            r#"<div style="font-family:sans-serif;max-width:600px;margin:0 auto">
  <div style="background:#161f3f;padding:24px 32px;border-radius:8px 8px 0 0">
    <h1 style="color:#fff;margin:0;font-size:20px">Voiture Voyages Fleet Management</h1>
  </div>
  <div style="padding:32px;border:1px solid #e5e7eb;border-top:none;border-radius:0 0 8px 8px">
    <p style="margin:0 0 16px">Hello <strong>{driver_name}</strong>,</p>
    <p style="margin:0 0 16px">Your <strong>{leave_label}</strong> request for <strong>{dates}</strong> has been
      <span style="color:{status_color};font-weight:600">{status}</span>.</p>
    {reason_block}
    <p style="margin:24px 0 0;color:#6b7280;font-size:13px">If you have any questions, please contact HR.</p>
  </div>
  <p style="color:#9ca3af;font-size:11px;text-align:center;margin:16px 0">© Voiture Voyages — Fleet Management System</p>
</div>"#
        );
        self.resend.send(to, &subject, &html).await
    }

    pub async fn send_invoice_email(
        &self,
        to: &str,
        driver_name: &str,
        invoice_no: &str,
        period: &str,
        total: &str,
    ) -> Result<(), AppError> {
        let subject = format!("Invoice {} — Voiture Voyages", invoice_no);
        let html = [
            "<div style='font-family:sans-serif;max-width:600px;margin:0 auto'>",
            "<div style='background:#161f3f;padding:24px 32px;border-radius:8px 8px 0 0'>",
            "<h1 style='color:#fff;margin:0;font-size:20px'>Voiture Voyages Fleet Management</h1></div>",
            "<div style='padding:32px;border:1px solid #e5e7eb;border-top:none;border-radius:0 0 8px 8px'>",
            &format!("<p>Hello <strong>{}</strong>,</p>", driver_name),
            "<p>Your invoice has been generated:</p>",
            &format!("<p><strong>Invoice:</strong> {}<br><strong>Period:</strong> {}<br><strong>Total:</strong> AED {}</p>", invoice_no, period, total),
            "<p style='color:#6b7280;font-size:13px'>Please retain this invoice for your records.</p>",
            "</div></div>",
        ].join("");
        self.resend.send(to, &subject, &html).await
    }

    fn capitalise(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
