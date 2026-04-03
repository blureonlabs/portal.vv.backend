use std::sync::Arc;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::common::{error::AppError, response::ApiResponse, types::CurrentUser};
use crate::advance::application::service::AdvanceService;
use crate::advance::presentation::dto::{
    AdvanceResponse, ListAdvancesQuery, PayAdvanceBody, RejectAdvanceBody, RequestAdvanceBody,
};
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
    notification_svc: web::Data<Arc<NotificationService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let advance = svc.approve(user.id, &user.role, *path).await?;
    // Fire-and-forget notification
    if let Some(email) = fetch_driver_email(db.pg_pool(), advance.driver_id).await {
        let amount = advance.amount_aed.to_string();
        notification_svc
            .send_advance_status_email(&email, &advance.driver_name, &amount, "approved", None)
            .await
            .ok();
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(AdvanceResponse::from(advance))))
}

pub async fn reject_advance(
    user: CurrentUser,
    svc: web::Data<Arc<AdvanceService>>,
    notification_svc: web::Data<Arc<NotificationService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    path: web::Path<Uuid>,
    body: web::Json<RejectAdvanceBody>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let body = body.into_inner();
    let rejection_reason = body.rejection_reason.clone();
    let advance = svc.reject(user.id, &user.role, *path, body.rejection_reason).await?;
    // Fire-and-forget notification
    if let Some(email) = fetch_driver_email(db.pg_pool(), advance.driver_id).await {
        let amount = advance.amount_aed.to_string();
        notification_svc
            .send_advance_status_email(
                &email,
                &advance.driver_name,
                &amount,
                "rejected",
                Some(&rejection_reason),
            )
            .await
            .ok();
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(AdvanceResponse::from(advance))))
}

pub async fn pay_advance(
    user: CurrentUser,
    svc: web::Data<Arc<AdvanceService>>,
    notification_svc: web::Data<Arc<NotificationService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    path: web::Path<Uuid>,
    body: web::Json<PayAdvanceBody>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let body = body.into_inner();
    let advance = svc
        .pay(user.id, &user.role, *path, body.payment_date, body.method, body.salary_period)
        .await?;
    // Fire-and-forget notification
    if let Some(email) = fetch_driver_email(db.pg_pool(), advance.driver_id).await {
        let amount = advance.amount_aed.to_string();
        notification_svc
            .send_advance_status_email(&email, &advance.driver_name, &amount, "paid", None)
            .await
            .ok();
    }
    Ok(HttpResponse::Ok().json(ApiResponse::ok(AdvanceResponse::from(advance))))
}
