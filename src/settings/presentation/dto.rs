use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::settings::domain::entity::Setting;

#[derive(Debug, Deserialize)]
pub struct UpdateSettingBody {
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub id: Uuid,
    pub key: String,
    pub value: String,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

impl From<Setting> for SettingResponse {
    fn from(s: Setting) -> Self {
        Self {
            id: s.id,
            key: s.key,
            value: s.value,
            updated_by: s.updated_by,
            updated_at: s.updated_at,
        }
    }
}
