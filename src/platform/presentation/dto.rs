use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreatePlatformRequest {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlatformRequest {
    pub name: String,
    pub is_active: bool,
    pub sort_order: i32,
}
