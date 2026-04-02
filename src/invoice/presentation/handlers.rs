use std::sync::Arc;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::common::{error::AppError, response::ApiResponse, types::CurrentUser};
use crate::invoice::application::service::InvoiceService;
use crate::invoice::domain::entity::LineItem;
use crate::invoice::presentation::dto::{
    GenerateInvoiceBody, InvoiceResponse, ListInvoicesQuery,
};

async fn resolve_driver_id(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<Uuid>, AppError> {
    let row = sqlx::query!("SELECT id FROM drivers WHERE profile_id = $1", user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.id))
}

pub async fn list_invoices(
    user: CurrentUser,
    svc: web::Data<Arc<InvoiceService>>,
    db: web::Data<crate::database::infrastructure::PgDatabase>,
    query: web::Query<ListInvoicesQuery>,
) -> Result<HttpResponse, AppError> {
    use crate::database::domain::DatabasePool;
    let actor_driver_id = resolve_driver_id(db.pg_pool(), user.id).await?;
    let invoices = svc.list(&user.role, actor_driver_id, query.driver_id).await?;
    let resp: Vec<InvoiceResponse> = invoices.into_iter().map(InvoiceResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn get_invoice(
    _user: CurrentUser,
    svc: web::Data<Arc<InvoiceService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let invoice = svc.find_by_id(*path).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(InvoiceResponse::from(invoice))))
}

pub async fn generate_invoice(
    user: CurrentUser,
    svc: web::Data<Arc<InvoiceService>>,
    body: web::Json<GenerateInvoiceBody>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    let line_items: Vec<LineItem> = body
        .line_items
        .into_iter()
        .map(|li| LineItem { description: li.description, amount_aed: li.amount_aed })
        .collect();

    let invoice = svc
        .generate(
            user.id,
            &user.role,
            body.driver_id,
            &body.driver_name,
            body.period_start,
            body.period_end,
            line_items,
        )
        .await?;

    Ok(HttpResponse::Created().json(ApiResponse::ok(InvoiceResponse::from(invoice))))
}
