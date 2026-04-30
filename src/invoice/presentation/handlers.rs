use std::sync::Arc;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::{ApiResponse, PaginatedResponse}, types::{CurrentUser, Role}};
use crate::invoice::application::service::InvoiceService;
use crate::invoice::domain::entity::LineItem;
use crate::invoice::presentation::dto::{
    GenerateInvoiceBody, InvoiceResponse, ListInvoicesQuery,
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

pub async fn list_invoices(
    user: CurrentUser,
    svc: web::Data<Arc<InvoiceService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    query: web::Query<ListInvoicesQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let actor_driver_id = resolve_driver_id(db.pg_pool(), user.id).await?;
    let all: Vec<InvoiceResponse> = svc
        .list(&user.role, actor_driver_id, query.driver_id)
        .await?
        .into_iter()
        .map(InvoiceResponse::from)
        .collect();
    let total = all.len() as i64;
    let page_data = all.into_iter().skip(offset as usize).take(limit as usize).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(PaginatedResponse::ok(page_data, page, limit, total)))
}

pub async fn get_invoice(
    user: CurrentUser,
    svc: web::Data<Arc<InvoiceService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let invoice = svc.find_by_id(*path).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(InvoiceResponse::from(invoice))))
}

pub async fn generate_invoice(
    user: CurrentUser,
    svc: web::Data<Arc<InvoiceService>>,
    notification_svc: web::Data<Arc<NotificationService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    body: web::Json<GenerateInvoiceBody>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let body = body.into_inner();
    let driver_id = body.driver_id;
    let line_items: Vec<LineItem> = body
        .line_items
        .into_iter()
        .map(|li| LineItem { description: li.description, amount_aed: li.amount_aed })
        .collect();

    let invoice = svc
        .generate(
            user.id,
            &user.role,
            driver_id,
            &body.driver_name,
            body.period_start,
            body.period_end,
            line_items,
        )
        .await?;

    // Fire-and-forget notification
    if let Some(email) = fetch_driver_email(db.pg_pool(), driver_id).await {
        let period = format!("{} to {}", invoice.period_start, invoice.period_end);
        let total = invoice.total_aed.to_string();
        notification_svc
            .send_invoice_email(
                &email,
                &invoice.driver_name,
                &invoice.invoice_no,
                &period,
                &total,
            )
            .await
            .ok();
    }

    Ok(HttpResponse::Created().json(ApiResponse::ok(InvoiceResponse::from(invoice))))
}
