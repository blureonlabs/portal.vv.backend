use std::sync::Arc;

use actix_web::{web, HttpResponse};

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::PaginatedResponse, types::{CurrentUser, Role}};
use crate::audit::application::service::AuditService;
use super::dto::{AuditEntryResponse, ListAuditQuery};

pub async fn list_audit(
    user: CurrentUser,
    svc: web::Data<Arc<AuditService>>,
    query: web::Query<ListAuditQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;

    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let (entries, total) = svc
        .list(
            query.entity_type.as_deref(),
            query.actor_id,
            query.action.as_deref(),
            limit,
            offset,
        )
        .await?;

    let resp: Vec<AuditEntryResponse> = entries.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(PaginatedResponse::ok(resp, page, limit, total)))
}
