use std::sync::Arc;
use actix_web::{web, HttpResponse};

use crate::auth::infrastructure::SupabaseAdminClient;
use crate::common::{response::ApiResponse, types::{CurrentUser, Role}};
use crate::auth::presentation::handlers::require_role;
use crate::owner::application::service::OwnerService;
use super::dto::*;

pub async fn list_owners(
    user: CurrentUser,
    svc: web::Data<Arc<OwnerService>>,
) -> Result<HttpResponse, crate::common::error::AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let owners: Vec<OwnerResponse> = svc.list().await?.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(owners)))
}

pub async fn get_owner(
    user: CurrentUser,
    svc: web::Data<Arc<OwnerService>>,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, crate::common::error::AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let owner: OwnerResponse = svc.find_by_id(path.into_inner()).await?.into();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(owner)))
}

pub async fn create_owner(
    user: CurrentUser,
    svc: web::Data<Arc<OwnerService>>,
    body: web::Json<CreateOwnerRequest>,
) -> Result<HttpResponse, crate::common::error::AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let b = body.into_inner();
    let owner: OwnerResponse = svc.create(b.profile_id, b.company_name.as_deref(), b.notes.as_deref()).await?.into();
    Ok(HttpResponse::Created().json(ApiResponse::ok(owner)))
}

pub async fn create_owner_with_account(
    user: CurrentUser,
    svc: web::Data<Arc<OwnerService>>,
    supabase: web::Data<Arc<SupabaseAdminClient>>,
    config: web::Data<crate::config::AppConfig>,
    auth_repo: web::Data<Arc<dyn crate::auth::domain::repository::AuthRepository>>,
    body: web::Json<CreateOwnerWithAccountRequest>,
) -> Result<HttpResponse, crate::common::error::AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let b = body.into_inner();

    // 1. Create Supabase auth user
    let user_id = supabase.create_user(&b.email, &b.password).await?;

    // 2. Create profile
    auth_repo.create_profile(user_id, &b.full_name, &b.email, &Role::Owner, b.phone.as_deref()).await?;

    // 3. Create owner record
    let owner: OwnerResponse = svc.create(user_id, b.company_name.as_deref(), b.notes.as_deref()).await?.into();

    Ok(HttpResponse::Created().json(ApiResponse::ok(owner)))
}

pub async fn update_owner(
    user: CurrentUser,
    svc: web::Data<Arc<OwnerService>>,
    path: web::Path<uuid::Uuid>,
    body: web::Json<UpdateOwnerRequest>,
) -> Result<HttpResponse, crate::common::error::AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let b = body.into_inner();
    let owner: OwnerResponse = svc.update(path.into_inner(), b.company_name.as_deref(), b.notes.as_deref()).await?.into();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(owner)))
}

/// Owner portal: get own data
pub async fn owner_me(
    user: CurrentUser,
    svc: web::Data<Arc<OwnerService>>,
) -> Result<HttpResponse, crate::common::error::AppError> {
    require_role(&user, &[Role::Owner])?;
    let owner: OwnerResponse = svc.find_by_profile_id(user.id).await?.into();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(owner)))
}
