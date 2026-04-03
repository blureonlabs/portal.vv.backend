use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::comms::domain::entity::*;

#[derive(Debug, Deserialize)]
pub struct SendBroadcastRequest {
    pub subject: String,
    pub body: String,
    pub channel: BroadcastChannel,
    pub target: BroadcastTarget,
    pub driver_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize)]
pub struct BroadcastResponse {
    pub id: Uuid,
    pub subject: String,
    pub body: String,
    pub channel: BroadcastChannel,
    pub target: BroadcastTarget,
    pub recipient_count: i32,
    pub status: BroadcastStatus,
    pub sent_by_name: String,
    pub created_at: String,
}

impl From<Broadcast> for BroadcastResponse {
    fn from(b: Broadcast) -> Self {
        Self {
            id: b.id,
            subject: b.subject,
            body: b.body,
            channel: b.channel,
            target: b.target,
            recipient_count: b.recipient_count,
            status: b.status,
            sent_by_name: b.sent_by_name,
            created_at: b.created_at.to_rfc3339(),
        }
    }
}
