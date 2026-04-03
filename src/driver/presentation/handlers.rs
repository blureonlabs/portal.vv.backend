use std::sync::Arc;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::auth::infrastructure::SupabaseAdminClient;
use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::driver::application::service::DriverService;
use crate::driver::presentation::dto::{
    CreateDriverRequest, CreateDriverWithAccountRequest, DriverEditResponse, DriverResponse,
    SetSelfEntryRequest, UpdateDriverRequest,
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
    supabase: web::Data<Arc<SupabaseAdminClient>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let driver_id = path.into_inner();
    let role = user.role.clone();

    // Fetch driver to get the Supabase auth user id (profile_id)
    let driver = svc.get(driver_id).await?;
    svc.activate(user.id, &role, driver_id).await?;

    // Re-enable the Supabase auth account (fire-and-forget; log error but don't fail)
    if let Err(e) = supabase.enable_user(driver.profile_id).await {
        tracing::error!("Failed to re-enable Supabase user {}: {}", driver.profile_id, e);
    }

    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn set_self_entry(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
    path: web::Path<Uuid>,
    body: web::Json<SetSelfEntryRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    svc.repo.set_self_entry(path.into_inner(), body.enabled).await?;
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

pub async fn create_driver_with_account(
    user: CurrentUser,
    svc: web::Data<Arc<DriverService>>,
    supabase: web::Data<Arc<SupabaseAdminClient>>,
    auth_repo: web::Data<Arc<dyn crate::auth::domain::repository::AuthRepository>>,
    body: web::Json<CreateDriverWithAccountRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let b = body.into_inner();

    // 1. Create Supabase auth user
    let user_id = supabase.create_user(&b.email, &b.password).await?;

    // 2. Create profile with role=driver and optional phone
    auth_repo.create_profile(user_id, &b.full_name, &b.email, &Role::Driver, b.phone.as_deref()).await?;

    // 3. Create driver record
    let driver: DriverResponse = svc.repo.create(user_id, &b.nationality, b.salary_type).await?.into();

    Ok(HttpResponse::Created().json(ApiResponse::ok(driver)))
}
