use std::sync::Arc;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::common::{error::AppError, response::ApiResponse, types::CurrentUser};
use crate::hr::application::service::HrService;
use crate::hr::presentation::dto::{LeaveResponse, ListLeaveQuery, RejectLeaveBody, SubmitLeaveBody};

async fn resolve_driver_id(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<Uuid>, AppError> {
    let row = sqlx::query!("SELECT id FROM drivers WHERE profile_id = $1", user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.id))
}

pub async fn list_leave(
    user: CurrentUser,
    svc: web::Data<Arc<HrService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    query: web::Query<ListLeaveQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let actor_driver_id = resolve_driver_id(db.pg_pool(), user.id).await?;
    let requests = svc
        .list(&user.role, actor_driver_id, query.driver_id, query.status.clone(), query.leave_type.clone())
        .await?;
    let resp: Vec<LeaveResponse> = requests.into_iter().map(LeaveResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
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
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let request = svc.approve(user.id, &user.role, *path).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(LeaveResponse::from(request))))
}

pub async fn reject_leave(
    user: CurrentUser,
    svc: web::Data<Arc<HrService>>,
    path: web::Path<Uuid>,
    body: web::Json<RejectLeaveBody>,
) -> Result<HttpResponse, AppError> {
    let request = svc.reject(user.id, &user.role, *path, body.into_inner().rejection_reason).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(LeaveResponse::from(request))))
}
