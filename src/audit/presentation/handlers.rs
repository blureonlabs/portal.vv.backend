use std::sync::Arc;

use actix_web::{web, HttpResponse};

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::audit::application::service::AuditService;
use super::dto::{AuditEntryResponse, ListAuditQuery};

pub async fn list_audit(
    user: CurrentUser,
    svc: web::Data<Arc<AuditService>>,
    query: web::Query<ListAuditQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;

    let entries = svc
        .list(
            query.entity_type.as_deref(),
            query.actor_id,
            query.action.as_deref(),
            query.limit,
            query.offset,
        )
        .await?;

    let resp: Vec<AuditEntryResponse> = entries.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}
