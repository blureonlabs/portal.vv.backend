use std::sync::Arc;
use actix_web::{web, HttpResponse};

use crate::common::{error::AppError, response::ApiResponse, types::{CurrentUser, Role}};
use crate::auth::presentation::handlers::require_role;
use crate::comms::application::service::CommsService;
use super::dto::*;

pub async fn list_broadcasts(
    user: CurrentUser,
    svc: web::Data<Arc<CommsService>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let broadcasts: Vec<BroadcastResponse> = svc.list().await?.into_iter().map(Into::into).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(broadcasts)))
}

pub async fn send_broadcast(
    user: CurrentUser,
    svc: web::Data<Arc<CommsService>>,
    body: web::Json<SendBroadcastRequest>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;
    let b = body.into_inner();
    let broadcast: BroadcastResponse = svc.send_broadcast(
        b.subject, b.body, b.channel, b.target, b.driver_ids, user.id
    ).await?.into();
    Ok(HttpResponse::Created().json(ApiResponse::ok(broadcast)))
}
