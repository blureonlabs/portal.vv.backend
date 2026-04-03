use std::sync::Arc;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::common::{error::AppError, response::ApiResponse, types::CurrentUser};
use crate::advance::application::service::AdvanceService;
use crate::advance::presentation::dto::{
    AdvanceResponse, ListAdvancesQuery, PayAdvanceBody, RejectAdvanceBody, RequestAdvanceBody,
};

async fn resolve_driver_id(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<Uuid>, AppError> {
    let row = sqlx::query!("SELECT id FROM drivers WHERE profile_id = $1", user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.id))
}

pub async fn list_advances(
    user: CurrentUser,
    svc: web::Data<Arc<AdvanceService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    query: web::Query<ListAdvancesQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let actor_driver_id = resolve_driver_id(db.pg_pool(), user.id).await?;
    let advances = svc.list(&user.role, actor_driver_id, query.driver_id, query.status.clone()).await?;
    let resp: Vec<AdvanceResponse> = advances.into_iter().map(AdvanceResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn request_advance(
    user: CurrentUser,
    svc: web::Data<Arc<AdvanceService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    body: web::Json<RequestAdvanceBody>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let body = body.into_inner();
    let actor_driver_id = resolve_driver_id(db.pg_pool(), user.id).await?;
    let advance = svc
        .request(user.id, &user.role, actor_driver_id, body.driver_id, body.amount_aed, body.reason)
        .await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(AdvanceResponse::from(advance))))
}

pub async fn approve_advance(
    user: CurrentUser,
    svc: web::Data<Arc<AdvanceService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let advance = svc.approve(user.id, &user.role, *path).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(AdvanceResponse::from(advance))))
}

pub async fn reject_advance(
    user: CurrentUser,
    svc: web::Data<Arc<AdvanceService>>,
    path: web::Path<Uuid>,
    body: web::Json<RejectAdvanceBody>,
) -> Result<HttpResponse, AppError> {
    let advance = svc.reject(user.id, &user.role, *path, body.into_inner().rejection_reason).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(AdvanceResponse::from(advance))))
}

pub async fn pay_advance(
    user: CurrentUser,
    svc: web::Data<Arc<AdvanceService>>,
    path: web::Path<Uuid>,
    body: web::Json<PayAdvanceBody>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    let advance = svc
        .pay(user.id, &user.role, *path, body.payment_date, body.method, body.salary_period)
        .await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(AdvanceResponse::from(advance))))
}
