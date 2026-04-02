use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: Option<T>,
    pub error: Option<String>,
    pub meta: Option<PaginationMeta>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { data: Some(data), error: None, meta: None }
    }

    pub fn paged(data: T, page: i64, page_size: i64, total: i64) -> Self {
        Self {
            data: Some(data),
            error: None,
            meta: Some(PaginationMeta { page, page_size, total }),
        }
    }
}

impl ApiResponse<()> {
    pub fn err(msg: impl Into<String>) -> Self {
        Self { data: None, error: Some(msg.into()), meta: None }
    }
}
