use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Shared pagination query params ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

impl PaginationQuery {
    /// Returns `(offset, limit)` clamped to sane values.
    pub fn offset_limit(&self) -> (i64, i64) {
        let limit = self.limit.unwrap_or(20).min(100).max(1);
        let page = self.page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        (offset, limit)
    }

    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum Role {
    SuperAdmin,
    Accountant,
    Hr,
    Driver,
    Owner,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "salary_type", rename_all = "snake_case")]
pub enum SalaryType {
    Commission,
    TargetHigh,
    TargetLow,
}

/// Injected into every authenticated handler via FromRequest extractor.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub role: Role,
    #[allow(dead_code)]
    pub email: String,
}
