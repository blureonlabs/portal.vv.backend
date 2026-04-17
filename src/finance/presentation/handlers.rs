use std::sync::Arc;

use actix_web::{web, HttpResponse};
use chrono::{Datelike, Local};
use uuid::Uuid;

use crate::common::{error::AppError, response::ApiResponse, types::CurrentUser};
use crate::finance::application::service::FinanceService;
use crate::finance::presentation::dto::{
    CreateExpenseRequest, CreateHandoverRequest, DateRangeQuery, ExpenseResponse, HandoverResponse,
};

/// Resolve the driver's own driver_id (for driver-role scoping). Finance module does a simpler
/// DB lookup since we don't store it in CurrentUser.
async fn resolve_driver_id_for_actor(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    let row = sqlx::query!("SELECT id FROM drivers WHERE profile_id = $1", user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.id))
}

pub async fn list_expenses(
    user: CurrentUser,
    svc: web::Data<Arc<FinanceService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let today = Local::now().date_naive();
    let from = query.from.unwrap_or_else(|| {
        chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today)
    });
    let to = query.to.unwrap_or(today);

    let actor_driver_id = resolve_driver_id_for_actor(db.pg_pool(), user.id).await?;
    let expenses = svc.list_expenses(&user.role, actor_driver_id, query.driver_id, from, to).await?;
    let resp: Vec<ExpenseResponse> = expenses.into_iter().map(ExpenseResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn create_expense(
    user: CurrentUser,
    svc: web::Data<Arc<FinanceService>>,
    body: web::Json<CreateExpenseRequest>,
) -> Result<HttpResponse, AppError> {
    use crate::common::validation::validate_amount;
    let body = body.into_inner();
    validate_amount("amount_aed", body.amount_aed)?;
    let expense = svc.create_expense(
        user.id,
        &user.role,
        body.driver_id,
        body.amount_aed,
        body.category,
        body.date,
        body.receipt_url,
        body.notes,
    ).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(ExpenseResponse::from(expense))))
}

pub async fn list_handovers(
    user: CurrentUser,
    svc: web::Data<Arc<FinanceService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    query: web::Query<DateRangeQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let today = Local::now().date_naive();
    let from = query.from.unwrap_or_else(|| {
        chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today)
    });
    let to = query.to.unwrap_or(today);

    let actor_driver_id = resolve_driver_id_for_actor(db.pg_pool(), user.id).await?;
    let handovers = svc.list_handovers(&user.role, actor_driver_id, query.driver_id, from, to).await?;
    let resp: Vec<HandoverResponse> = handovers.into_iter().map(HandoverResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn create_handover(
    user: CurrentUser,
    svc: web::Data<Arc<FinanceService>>,
    body: web::Json<CreateHandoverRequest>,
) -> Result<HttpResponse, AppError> {
    use crate::common::validation::validate_amount;
    let body = body.into_inner();
    validate_amount("amount_aed", body.amount_aed)?;
    let handover = svc.create_handover(user.id, &user.role, body.driver_id, body.amount_aed).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(HandoverResponse::from(handover))))
}
