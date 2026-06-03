use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::platform::domain::{
    entity::{CreatePlatform, UpdatePlatform},
    repository::PlatformRepository,
};
use super::dto::{CreatePlatformRequest, UpdatePlatformRequest};

pub async fn list_platforms(
    user: CurrentUser,
    repo: web::Data<Arc<dyn PlatformRepository>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let platforms = repo.list_active().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(platforms)))
}

pub async fn create_platform(
    user: CurrentUser,
    repo: web::Data<Arc<dyn PlatformRepository>>,
    body: web::Json<CreatePlatformRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let name = body.name.trim().to_string();
    let code = body.code.trim().to_string();
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::BadRequest("Name must be 1-100 characters".into()));
    }
    if code.is_empty() || code.len() > 50 || !code.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(AppError::BadRequest("Code must be 1-50 lowercase alphanumeric characters with underscores".into()));
    }
    let platform = repo.create(CreatePlatform { name, code }).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(platform)))
}

pub async fn update_platform(
    user: CurrentUser,
    repo: web::Data<Arc<dyn PlatformRepository>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdatePlatformRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let id = path.into_inner();
    let name = body.name.trim().to_string();
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::BadRequest("Name must be 1-100 characters".into()));
    }
    let platform = repo.update(id, UpdatePlatform {
        name,
        is_active: body.is_active,
        sort_order: body.sort_order,
    }).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(platform)))
}

pub async fn deactivate_platform(
    user: CurrentUser,
    repo: web::Data<Arc<dyn PlatformRepository>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let id = path.into_inner();
    repo.deactivate(id).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok("Deactivated")))
}
