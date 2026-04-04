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

    #[allow(dead_code)]
    pub fn paged(data: T, page: i64, page_size: i64, total: i64) -> Self {
        Self {
            data: Some(data),
            error: None,
            meta: Some(PaginationMeta { page, page_size, total }),
        }
    }
}

impl ApiResponse<()> {
    #[allow(dead_code)]
    pub fn err(msg: impl Into<String>) -> Self {
        Self { data: None, error: Some(msg.into()), meta: None }
    }
}

// ── Paginated list response ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub error: Option<String>,
    pub meta: ListPaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct ListPaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn ok(data: Vec<T>, page: i64, limit: i64, total: i64) -> Self {
        Self {
            data,
            error: None,
            meta: ListPaginationMeta { page, limit, total },
        }
    }
}
