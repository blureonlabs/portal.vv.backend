use std::sync::Arc;
use actix_web::{web, HttpResponse};
use crate::common::{error::AppError, response::ApiResponse, types::CurrentUser};
use crate::platform::domain::repository::PlatformRepository;

pub async fn list_platforms(
    _user: CurrentUser,
    repo: web::Data<Arc<dyn PlatformRepository>>,
) -> Result<HttpResponse, AppError> {
    let platforms = repo.list_active().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(platforms)))
}
