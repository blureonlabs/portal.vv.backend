use std::sync::Arc;

use actix_web::{web, HttpResponse};

use crate::auth::presentation::handlers::require_role;
use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::settings::application::service::SettingsService;
use super::dto::{SettingResponse, UpdateSettingBody};

pub async fn list_settings(
    user: CurrentUser,
    svc: web::Data<Arc<SettingsService>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin, Role::Accountant])?;
    let settings = svc.list().await?;
    let resp: Vec<SettingResponse> = settings.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(resp)))
}

pub async fn update_setting(
    user: CurrentUser,
    svc: web::Data<Arc<SettingsService>>,
    key: web::Path<String>,
    body: web::Json<UpdateSettingBody>,
) -> Result<HttpResponse, AppError> {
    match user.role {
        Role::SuperAdmin | Role::Accountant => {}
        _ => return Err(AppError::Forbidden("Only super_admin or accountant can update settings".into())),
    }
    let setting = svc.update(user.id, &user.role, &key, &body.value).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(SettingResponse::from(setting))))
}
