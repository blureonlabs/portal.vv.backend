use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "broadcast_channel", rename_all = "snake_case")]
pub enum BroadcastChannel { Email, Whatsapp }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "broadcast_target", rename_all = "snake_case")]
pub enum BroadcastTarget { AllDrivers, SelectedDrivers }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "broadcast_status", rename_all = "snake_case")]
pub enum BroadcastStatus { Draft, Sending, Sent, Failed }

#[derive(Debug, Clone)]
pub struct Broadcast {
    pub id: Uuid,
    pub subject: String,
    pub body: String,
    pub channel: BroadcastChannel,
    pub target: BroadcastTarget,
    pub target_driver_ids: Option<Vec<Uuid>>,
    pub sent_by: Uuid,
    pub sent_by_name: String,
    pub recipient_count: i32,
    pub status: BroadcastStatus,
    pub created_at: DateTime<Utc>,
}
