use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::deserialize::empty_string_as_none_string;
use crate::owner::domain::entity::Owner;

#[derive(Debug, Deserialize)]
pub struct CreateOwnerRequest {
    pub profile_id: Uuid,
    #[serde(default, deserialize_with = "empty_string_as_none_string")]
    pub company_name: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none_string")]
    pub notes: Option<String>,
}

/// Create owner with account in one step (admin use)
#[derive(Debug, Deserialize)]
pub struct CreateOwnerWithAccountRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    #[serde(default, deserialize_with = "empty_string_as_none_string")]
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none_string")]
    pub company_name: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none_string")]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOwnerRequest {
    #[serde(default, deserialize_with = "empty_string_as_none_string")]
    pub company_name: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none_string")]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OwnerResponse {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

impl From<Owner> for OwnerResponse {
    fn from(o: Owner) -> Self {
        Self {
            id: o.id,
            profile_id: o.profile_id,
            full_name: o.full_name,
            email: o.email,
            phone: o.phone,
            company_name: o.company_name,
            notes: o.notes,
            is_active: o.is_active,
            created_at: o.created_at.to_rfc3339(),
        }
    }
}
