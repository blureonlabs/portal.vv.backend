use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::driver::application::service::DriverService;
use crate::driver::presentation::dto::{
    CreateDriverRequest, DriverEditResponse, DriverResponse, UpdateDriverRequest,
};

pub async fn list_drivers(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let drivers: Vec<DriverResponse> = svc.list().await?.into_iter().map(DriverResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(drivers)))
}

pub async fn get_driver(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant, Role::Hr])?;
    let driver = svc.get(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(DriverResponse::from(driver))))
}

pub async fn create_driver(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
    body: web::Json<CreateDriverRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let body = body.into_inner();
    let role = user.role.clone();
    let driver = svc.create(user.id, &role, body.profile_id, body.nationality, body.salary_type).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(DriverResponse::from(driver))))
}

pub async fn update_driver(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateDriverRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let body = body.into_inner();
    let role = user.role.clone();
    let driver = svc.update(user.id, &role, path.into_inner(), body.nationality, body.salary_type).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(DriverResponse::from(driver))))
}

pub async fn deactivate_driver(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let role = user.role.clone();
    svc.deactivate(user.id, &role, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn activate_driver(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let role = user.role.clone();
    svc.activate(user.id, &role, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn list_driver_edits(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let edits: Vec<DriverEditResponse> = svc.repo.list_edits(path.into_inner()).await?
        .into_iter().map(DriverEditResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(edits)))
}
