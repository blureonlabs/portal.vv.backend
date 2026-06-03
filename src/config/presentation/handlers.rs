use std::sync::Arc;

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::config::application::service::ConfigService;
use super::dto::{ActiveOnlyQuery, CreateConfigItemRequest, CreateDocumentTypeRequest, UpdateConfigItemRequest};

// ── Expense categories ──────────────────────────────────────────────────────

pub async fn list_expense_categories(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    query: web::Query<ActiveOnlyQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let active_only = query.active_only.unwrap_or(true);
    let items = svc.list_expense_categories(active_only).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

pub async fn create_expense_category(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    body: web::Json<CreateConfigItemRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let item = svc.create_expense_category(&body.name, &body.code, user.id, &user.role).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(item)))
}

pub async fn update_expense_category(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateConfigItemRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let id = path.into_inner();
    svc.update_expense_category(id, &body.name, body.is_active, body.sort_order, user.id, &user.role).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok("Updated")))
}

pub async fn delete_expense_category(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let id = path.into_inner();
    svc.delete_expense_category(id, user.id, &user.role).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok("Deleted")))
}

// ── Leave types ─────────────────────────────────────────────────────────────

pub async fn list_leave_types(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    query: web::Query<ActiveOnlyQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let active_only = query.active_only.unwrap_or(true);
    let items = svc.list_leave_types(active_only).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

pub async fn create_leave_type(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    body: web::Json<CreateConfigItemRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let item = svc.create_leave_type(&body.name, &body.code, user.id, &user.role).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(item)))
}

pub async fn update_leave_type(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateConfigItemRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let id = path.into_inner();
    svc.update_leave_type(id, &body.name, body.is_active, body.sort_order, user.id, &user.role).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok("Updated")))
}

pub async fn delete_leave_type(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let id = path.into_inner();
    svc.delete_leave_type(id, user.id, &user.role).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok("Deleted")))
}

// ── Document types ──────────────────────────────────────────────────────────

pub async fn list_document_types(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    query: web::Query<ActiveOnlyQuery>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let active_only = query.active_only.unwrap_or(true);
    let items = svc.list_document_types(active_only).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

pub async fn create_document_type(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    body: web::Json<CreateDocumentTypeRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let item = svc.create_document_type(&body.name, &body.code, &body.applies_to, user.id, &user.role).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(item)))
}

pub async fn update_document_type(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateConfigItemRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let id = path.into_inner();
    svc.update_document_type(id, &body.name, body.is_active, body.sort_order, user.id, &user.role).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok("Updated")))
}

pub async fn delete_document_type(
    user: CurrentUser,
    svc: web::Data<Arc<ConfigService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let id = path.into_inner();
    svc.delete_document_type(id, user.id, &user.role).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok("Deleted")))
}
