use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateConfigItemRequest {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigItemRequest {
    pub name: String,
    pub is_active: bool,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentTypeRequest {
    pub name: String,
    pub code: String,
    pub applies_to: String,
}

#[derive(Debug, Deserialize)]
pub struct ActiveOnlyQuery {
    pub active_only: Option<bool>,
}
