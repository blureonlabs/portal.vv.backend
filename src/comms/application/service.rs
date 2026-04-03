use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::AppError;
use crate::comms::domain::entity::*;
use crate::comms::infrastructure::PgCommsRepository;
use crate::notification::application::service::NotificationService;

pub struct CommsService {
    repo: PgCommsRepository,
    notification: Arc<NotificationService>,
}

impl CommsService {
    pub fn new(repo: PgCommsRepository, notification: Arc<NotificationService>) -> Self {
        Self { repo, notification }
    }

    pub async fn list(&self) -> Result<Vec<Broadcast>, AppError> {
        self.repo.list().await
    }

    pub async fn send_broadcast(
        &self,
        subject: String,
        body: String,
        channel: BroadcastChannel,
        target: BroadcastTarget,
        target_driver_ids: Option<Vec<Uuid>>,
        sent_by: Uuid,
    ) -> Result<Broadcast, AppError> {
        // WhatsApp not yet supported
        if channel == BroadcastChannel::Whatsapp {
            return Err(AppError::BadRequest("WhatsApp integration coming soon".into()));
        }

        let ids_slice = target_driver_ids.as_deref();

        // Create broadcast record
        let broadcast = self.repo.create(
            &subject, &body, &channel, &target, ids_slice, sent_by
        ).await?;

        // Get recipient emails
        let recipients = self.repo.get_driver_emails(ids_slice).await?;
        let count = recipients.len() as i32;

        // Update status to sending
        self.repo.update_status(broadcast.id, &BroadcastStatus::Sending, 0).await?;

        // Send emails
        let mut sent = 0;
        for (_name, email) in &recipients {
            match self.notification.send_broadcast_email(email, &subject, &body).await {
                Ok(_) => sent += 1,
                Err(e) => tracing::warn!("Failed to send broadcast to {}: {}", email, e),
            }
        }

        // Update final status
        let final_status = if sent == count { BroadcastStatus::Sent } else if sent > 0 { BroadcastStatus::Sent } else { BroadcastStatus::Failed };
        self.repo.update_status(broadcast.id, &final_status, sent).await?;

        // Re-fetch to return updated record
        let updated = self.repo.list().await?.into_iter().find(|b| b.id == broadcast.id)
            .ok_or_else(|| AppError::Internal("Broadcast not found after send".into()))?;
        Ok(updated)
    }
}
