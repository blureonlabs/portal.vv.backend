use std::sync::Arc;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::common::{error::AppError, response::{ApiResponse, PaginatedResponse}, types::CurrentUser};
use crate::hr::application::service::HrService;
use crate::hr::presentation::dto::{BulkApproveBody, BulkApproveResponse, LeaveResponse, ListLeaveQuery, RejectLeaveBody, SubmitLeaveBody};
use crate::notification::application::service::NotificationService;

async fn resolve_driver_id(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<Uuid>, AppError> {
    let row = sqlx::query!("SELECT id FROM drivers WHERE profile_id = $1", user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.id))
}

/// Fetch the driver's email from profiles via the drivers table.
async fn fetch_driver_email(pool: &sqlx::PgPool, driver_id: Uuid) -> Option<String> {
    sqlx::query!(
        "SELECT p.email FROM profiles p
         JOIN drivers d ON d.profile_id = p.id
         WHERE d.id = $1",
        driver_id
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .map(|r| r.email)
}

pub async fn list_leave(
    user: CurrentUser,
    svc: web::Data<Arc<HrService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    query: web::Query<ListLeaveQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let actor_driver_id = resolve_driver_id(db.pg_pool(), user.id).await?;
    let all: Vec<LeaveResponse> = svc
        .list(&user.role, actor_driver_id, query.driver_id, query.status.clone(), query.leave_type.clone())
        .await?
        .into_iter()
        .map(LeaveResponse::from)
        .collect();
    let total = all.len() as i64;
    let page_data = all.into_iter().skip(offset as usize).take(limit as usize).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(PaginatedResponse::ok(page_data, page, limit, total)))
}

pub async fn submit_leave(
    user: CurrentUser,
    svc: web::Data<Arc<HrService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    body: web::Json<SubmitLeaveBody>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let body = body.into_inner();
    let actor_driver_id = resolve_driver_id(db.pg_pool(), user.id).await?;
    let request = svc
        .submit(
            user.id,
            &user.role,
            actor_driver_id,
            body.driver_id,
            body.leave_type,
            body.from_date,
            body.to_date,
            body.reason,
        )
        .await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(LeaveResponse::from(request))))
}

pub async fn approve_leave(
    user: CurrentUser,
    svc: web::Data<Arc<HrService>>,
    notification_svc: web::Data<Arc<NotificationService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let request = svc.approve(user.id, &user.role, *path).await?;
    // Fire-and-forget notification
    if let Some(email) = fetch_driver_email(db.pg_pool(), request.driver_id).await {
        let leave_type = format!("{:?}", request.r#type).to_lowercase();
        let dates = format!("{} to {}", request.from_date, request.to_date);
        notification_svc
            .send_leave_status_email(
                &email,
                &request.driver_name,
                &leave_type,
                &dates,
                "approved",
                None,
            )
            .await
            .ok();
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(LeaveResponse::from(request))))
}

pub async fn bulk_approve_leave(
    user: CurrentUser,
    svc: web::Data<Arc<HrService>>,
    body: web::Json<BulkApproveBody>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    let approved_count = svc.bulk_approve(user.id, &user.role, body.request_ids).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(BulkApproveResponse { approved_count })))
}

pub async fn reject_leave(
    user: CurrentUser,
    svc: web::Data<Arc<HrService>>,
    notification_svc: web::Data<Arc<NotificationService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    path: web::Path<Uuid>,
    body: web::Json<RejectLeaveBody>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let body = body.into_inner();
    let rejection_reason = body.rejection_reason.clone();
    let request = svc.reject(user.id, &user.role, *path, body.rejection_reason).await?;
    // Fire-and-forget notification
    if let Some(email) = fetch_driver_email(db.pg_pool(), request.driver_id).await {
        let leave_type = format!("{:?}", request.r#type).to_lowercase();
        let dates = format!("{} to {}", request.from_date, request.to_date);
        notification_svc
            .send_leave_status_email(
                &email,
                &request.driver_name,
                &leave_type,
                &dates,
                "rejected",
                Some(&rejection_reason),
            )
            .await
            .ok();
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(LeaveResponse::from(request))))
}
