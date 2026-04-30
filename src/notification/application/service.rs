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

    // ── Shared branded template ──────────────────────────────────────────────

    fn email_template(title: &str, body_html: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body style="margin:0;padding:0;background:#f4f5f7;font-family:Arial,Helvetica,sans-serif">
<table width="100%" cellpadding="0" cellspacing="0" style="background:#f4f5f7;padding:32px 0">
<tr><td align="center">
<table width="600" cellpadding="0" cellspacing="0" style="max-width:600px;width:100%">

  <!-- Header -->
  <tr><td style="background:#161f3f;padding:32px;text-align:center;border-radius:12px 12px 0 0">
    <img src="https://ohigikaddfxqvrcpxiwj.supabase.co/storage/v1/object/public/vv-images/logo.png"
         alt="Voiture Voyages" height="48"
         style="display:block;margin:0 auto 8px">
    <p style="color:#ffffff;margin:0;font-size:13px;opacity:0.75;letter-spacing:0.5px">Fleet Management System</p>
  </td></tr>

  <!-- Body -->
  <tr><td style="background:#ffffff;padding:40px 32px;border-left:1px solid #e5e7eb;border-right:1px solid #e5e7eb">
    <h2 style="color:#161f3f;margin:0 0 24px;font-size:20px;font-weight:700">{title}</h2>
    {body_html}
  </td></tr>

  <!-- Footer -->
  <tr><td style="background:#f9fafb;padding:24px 32px;border:1px solid #e5e7eb;border-top:none;border-radius:0 0 12px 12px;text-align:center">
    <p style="color:#6b7280;margin:0 0 6px;font-size:12px">Voiture Voyages — Fleet Management System</p>
    <p style="color:#9ca3af;margin:0;font-size:11px">This is an automated message. Please do not reply.</p>
  </td></tr>

</table>
</td></tr>
</table>
</body>
</html>"#,
            title = title,
            body_html = body_html
        )
    }

    // ── Status badge helper ──────────────────────────────────────────────────

    fn status_badge(status: &str) -> String {
        let (bg, fg) = match status {
            "approved" => ("#dcfce7", "#166534"),
            "rejected" => ("#fef2f2", "#991b1b"),
            "paid"     => ("#eff6ff", "#1e40af"),
            "pending"  => ("#fefce8", "#854d0e"),
            _          => ("#f3f4f6", "#374151"),
        };
        format!(
            r#"<span style="display:inline-block;padding:4px 14px;border-radius:50px;font-size:12px;font-weight:700;background:{bg};color:{fg}">{label}</span>"#,
            bg = bg,
            fg = fg,
            label = Self::capitalise(status),
        )
    }

    // ── CTA button helper ────────────────────────────────────────────────────

    fn cta_button(url: &str, label: &str) -> String {
        format!(
            r#"<a href="{url}" style="display:inline-block;background:#161f3f;color:#ffffff;padding:12px 32px;border-radius:50px;text-decoration:none;font-weight:700;font-size:14px;letter-spacing:0.3px">{label}</a>"#,
            url = url,
            label = label,
        )
    }

    // ── Public email methods ─────────────────────────────────────────────────

    pub async fn send_invite_email(
        &self,
        to_email: &str,
        full_name: &str,
        invite_url: &str,
    ) -> Result<(), AppError> {
        let subject = "You've Been Invited to Voiture Voyages FMS";
        let button = Self::cta_button(invite_url, "Accept Invitation");
        let body = format!(
            r#"<p style="color:#374151;margin:0 0 16px;font-size:15px;line-height:1.6">
  Hello <strong>{full_name}</strong>,
</p>
<p style="color:#374151;margin:0 0 24px;font-size:15px;line-height:1.6">
  You have been invited to join the <strong>Voiture Voyages Fleet Management System</strong>.
  Click the button below to set up your account and get started.
</p>
<p style="margin:0 0 32px;text-align:center">
  {button}
</p>
<p style="color:#6b7280;margin:0;font-size:13px;line-height:1.5">
  This invitation link expires in <strong>24 hours</strong>. If you did not expect this invitation, you can safely ignore this email.
</p>"#
        );
        let html = Self::email_template("You've Been Invited", &body);
        self.resend.send(to_email, subject, &html).await
    }

    pub async fn send_broadcast_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), AppError> {
        let body = format!(
            r#"<div style="color:#374151;font-size:15px;line-height:1.7">
  {html_body}
</div>"#
        );
        let html = Self::email_template(subject, &body);
        self.resend.send(to_email, subject, &html).await
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_url: &str,
    ) -> Result<(), AppError> {
        let subject = "Reset Your Password";
        let button = Self::cta_button(reset_url, "Reset Password");
        let body = format!(
            r#"<p style="color:#374151;margin:0 0 16px;font-size:15px;line-height:1.6">
  Hello,
</p>
<p style="color:#374151;margin:0 0 24px;font-size:15px;line-height:1.6">
  We received a request to reset the password for your FMS account. Click the button below to choose a new password.
</p>
<p style="margin:0 0 32px;text-align:center">
  {button}
</p>
<p style="color:#6b7280;margin:0;font-size:13px;line-height:1.5">
  This link expires in <strong>1 hour</strong>. If you did not request a password reset, you can safely ignore this email — your account remains secure.
</p>"#
        );
        let html = Self::email_template("Reset Your Password", &body);
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
        let title = format!("Advance Request {}", Self::capitalise(status));
        let subject = format!("Advance Request {} — Voiture Voyages", Self::capitalise(status));
        let badge = Self::status_badge(status);
        let reason_block = reason
            .map(|r| format!(
                r#"<div style="margin:24px 0;padding:16px;background:#f9fafb;border-left:4px solid #e5e7eb;border-radius:4px">
  <p style="margin:0;color:#374151;font-size:14px"><strong>Reason:</strong> {r}</p>
</div>"#
            ))
            .unwrap_or_default();
        let body = format!(
            r#"<p style="color:#374151;margin:0 0 16px;font-size:15px;line-height:1.6">
  Hello <strong>{driver_name}</strong>,
</p>
<p style="color:#374151;margin:0 0 24px;font-size:15px;line-height:1.6">
  Your advance request has been reviewed. Here are the details:
</p>
<table cellpadding="0" cellspacing="0" style="width:100%;border-collapse:collapse;margin:0 0 24px">
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280;width:40%">Amount Requested</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb;font-size:15px;font-weight:700;color:#161f3f">AED {amount}</td>
  </tr>
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280">Status</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb">{badge}</td>
  </tr>
</table>
{reason_block}
<p style="color:#6b7280;margin:0;font-size:13px;line-height:1.5">
  If you have any questions, please contact the operations team.
</p>"#
        );
        let html = Self::email_template(&title, &body);
        self.resend.send(to, &subject, &html).await
    }

    pub async fn send_advance_request_notification(
        &self,
        to: &str,
        driver_name: &str,
        amount: &str,
    ) -> Result<(), AppError> {
        let subject = "New Advance Request — Action Required";
        let body = format!(
            r#"<p style="color:#374151;margin:0 0 16px;font-size:15px;line-height:1.6">
  A new advance request has been submitted and requires your review.
</p>
<table cellpadding="0" cellspacing="0" style="width:100%;border-collapse:collapse;margin:0 0 32px">
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280;width:40%">Driver</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb;font-size:15px;font-weight:600;color:#161f3f">{driver_name}</td>
  </tr>
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280">Amount Requested</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb;font-size:15px;font-weight:700;color:#161f3f">AED {amount}</td>
  </tr>
</table>
<p style="color:#6b7280;margin:0;font-size:13px;line-height:1.5">
  Please log in to the Fleet Management System to review and action this request.
</p>"#
        );
        let html = Self::email_template("New Advance Request", &body);
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
        let title = format!("Leave Request {}", Self::capitalise(status));
        let subject = format!("Leave Request {} — Voiture Voyages", Self::capitalise(status));
        let badge = Self::status_badge(status);
        let leave_label = Self::capitalise(leave_type);
        let reason_block = reason
            .map(|r| format!(
                r#"<div style="margin:24px 0;padding:16px;background:#f9fafb;border-left:4px solid #e5e7eb;border-radius:4px">
  <p style="margin:0;color:#374151;font-size:14px"><strong>Reason:</strong> {r}</p>
</div>"#
            ))
            .unwrap_or_default();
        let body = format!(
            r#"<p style="color:#374151;margin:0 0 16px;font-size:15px;line-height:1.6">
  Hello <strong>{driver_name}</strong>,
</p>
<p style="color:#374151;margin:0 0 24px;font-size:15px;line-height:1.6">
  Your leave request has been reviewed. Here are the details:
</p>
<table cellpadding="0" cellspacing="0" style="width:100%;border-collapse:collapse;margin:0 0 24px">
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280;width:40%">Leave Type</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb;font-size:15px;font-weight:600;color:#161f3f">{leave_label}</td>
  </tr>
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280">Dates</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb;font-size:15px;color:#374151">{dates}</td>
  </tr>
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280">Status</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb">{badge}</td>
  </tr>
</table>
{reason_block}
<p style="color:#6b7280;margin:0;font-size:13px;line-height:1.5">
  If you have any questions, please contact HR or your operations manager.
</p>"#
        );
        let html = Self::email_template(&title, &body);
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
        let title = format!("Invoice {}", invoice_no);
        let subject = format!("Invoice {} — Voiture Voyages", invoice_no);
        let body = format!(
            r#"<p style="color:#374151;margin:0 0 16px;font-size:15px;line-height:1.6">
  Hello <strong>{driver_name}</strong>,
</p>
<p style="color:#374151;margin:0 0 24px;font-size:15px;line-height:1.6">
  Your invoice has been generated. Please find the details below.
</p>
<table cellpadding="0" cellspacing="0" style="width:100%;border-collapse:collapse;margin:0 0 32px">
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280;width:40%">Invoice Number</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb;font-size:15px;font-weight:700;color:#161f3f">{invoice_no}</td>
  </tr>
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280">Period</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb;font-size:15px;color:#374151">{period}</td>
  </tr>
  <tr>
    <td style="padding:12px 16px;background:#f9fafb;border:1px solid #e5e7eb;font-size:13px;color:#6b7280">Total Amount</td>
    <td style="padding:12px 16px;background:#ffffff;border:1px solid #e5e7eb;font-size:18px;font-weight:700;color:#161f3f">AED {total}</td>
  </tr>
</table>
<p style="color:#6b7280;margin:0;font-size:13px;line-height:1.5">
  Please retain this invoice for your records.
</p>"#
        );
        let html = Self::email_template(&title, &body);
        self.resend.send(to, &subject, &html).await
    }

    // ── Utilities ────────────────────────────────────────────────────────────

    fn capitalise(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None    => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
