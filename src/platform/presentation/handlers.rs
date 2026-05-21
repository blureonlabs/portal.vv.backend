use std::sync::Arc;
use actix_web::{web, HttpResponse};
use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::platform::domain::repository::PlatformRepository;

pub async fn list_platforms(
    user: CurrentUser,
    repo: web::Data<Arc<dyn PlatformRepository>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let platforms = repo.list_active().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(platforms)))
}
