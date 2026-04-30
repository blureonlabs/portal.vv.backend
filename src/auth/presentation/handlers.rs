use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use actix_web::{dev::Payload, web, FromRequest, HttpRequest, HttpResponse};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::application::service::AuthService;
use crate::auth::domain::repository::AuthRepository;
use crate::auth::presentation::dto::{
    AcceptInviteRequest, ForgotPasswordRequest, InviteResponse, InviteUserRequest, MeResponse,
    ResetPasswordRequest, UpdateAvatarRequest, UserResponse,
};
use crate::common::{
    error::AppError,
    response::ApiResponse,
    types::{CurrentUser, Role},
    validation::validate_password,
};

// ── JWT Claims ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SupabaseClaims {
    sub: String,
    email: Option<String>,
}

// ── CurrentUser extractor ─────────────────────────────────────────────────────

impl FromRequest for CurrentUser {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<CurrentUser, AppError>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Clone everything we need before entering the async block
        let config = req
            .app_data::<web::Data<crate::config::AppConfig>>()
            .cloned();
        let repo = req
            .app_data::<web::Data<Arc<dyn AuthRepository>>>()
            .cloned();
        let raw_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        Box::pin(async move {
            let config = config.ok_or_else(|| AppError::Internal("Config missing".into()))?;
            let repo = repo.ok_or_else(|| AppError::Internal("Repo missing".into()))?;
            let header = raw_header.ok_or(AppError::Unauthorized)?;

            let token = header
                .strip_prefix("Bearer ")
                .ok_or(AppError::Unauthorized)?
                .to_owned();

            // Try ES256 (newer Supabase projects) first, fall back to HS256
            let claims = {
                // ES256: fetch JWKS components from config
                let es256_result = (|| -> Result<SupabaseClaims, ()> {
                    let jwks_x = std::env::var("SUPABASE_JWKS_X").map_err(|_| ())?;
                    let jwks_y = std::env::var("SUPABASE_JWKS_Y").map_err(|_| ())?;
                    let key = DecodingKey::from_ec_components(&jwks_x, &jwks_y).map_err(|_| ())?;
                    let mut v = Validation::new(Algorithm::ES256);
                    v.set_audience(&["authenticated"]);
                    decode::<SupabaseClaims>(&token, &key, &v).map(|d| d.claims).map_err(|_| ())
                })();

                if let Ok(c) = es256_result {
                    c
                } else {
                    // HS256 fallback (older Supabase projects)
                    tracing::warn!("JWT ES256 validation failed or JWKS vars not set, falling back to HS256");
                    let key = DecodingKey::from_secret(config.supabase_jwt_secret.as_bytes());
                    let mut v = Validation::new(Algorithm::HS256);
                    v.set_audience(&["authenticated"]);
                    decode::<SupabaseClaims>(&token, &key, &v)
                        .map_err(|_| AppError::Unauthorized)?
                        .claims
                }
            };

            let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

            let profile = repo
                .find_profile_by_id(user_id)
                .await?
                .ok_or(AppError::Unauthorized)?;

            if !profile.is_active {
                return Err(AppError::Forbidden("Account is deactivated".into()));
            }

            Ok(CurrentUser {
                id: user_id,
                role: profile.role,
                email: claims.email.unwrap_or_default(),
            })
        })
    }
}

// ── Role guard ────────────────────────────────────────────────────────────────

pub fn require_role(user: &CurrentUser, allowed: &[Role]) -> Result<(), AppError> {
    if allowed.contains(&user.role) {
        Ok(())
    } else {
        Err(AppError::Forbidden("Insufficient permissions".into()))
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn me(
    user: CurrentUser,
    svc: web::Data<Arc<AuthService>>,
) -> Result<HttpResponse, AppError> {
    let profile = svc
        .repo
        .find_profile_by_id(user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found".into()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::ok(MeResponse::from(profile))))
}

pub async fn list_users(
    user: CurrentUser,
    svc: web::Data<Arc<AuthService>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;

    let profiles = svc.repo.list_profiles().await?;
    let data: Vec<UserResponse> = profiles.into_iter().map(UserResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(data)))
}

pub async fn list_invites(
    user: CurrentUser,
    svc: web::Data<Arc<AuthService>>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;

    let invites = svc.repo.list_invites().await?;
    let data: Vec<InviteResponse> = invites.into_iter().map(InviteResponse::from).collect();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(data)))
}

pub async fn invite_user(
    user: CurrentUser,
    svc: web::Data<Arc<AuthService>>,
    body: web::Json<InviteUserRequest>,
) -> Result<HttpResponse, AppError> {
    use crate::common::validation::validate_string_length;
    require_role(&user, &[Role::SuperAdmin])?;

    let body = body.into_inner();
    validate_string_length("full_name", &body.full_name, 200)?;
    validate_string_length("email", &body.email, 254)?;
    let actor_role = user.role.clone();
    let invite_id = svc
        .invite_user(user.id, &actor_role, body.email, body.full_name, body.role)
        .await?;

    Ok(HttpResponse::Created().json(ApiResponse::ok(serde_json::json!({ "id": invite_id }))))
}

pub async fn revoke_invite(
    user: CurrentUser,
    svc: web::Data<Arc<AuthService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;

    let role = user.role.clone();
    svc.revoke_invite(user.id, &role, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn resend_invite(
    user: CurrentUser,
    svc: web::Data<Arc<AuthService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    require_role(&user, &[Role::SuperAdmin])?;

    let role = user.role.clone();
    svc.resend_invite(user.id, &role, path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn accept_invite(
    svc: web::Data<Arc<AuthService>>,
    body: web::Json<AcceptInviteRequest>,
) -> Result<HttpResponse, AppError> {
    let body = body.into_inner();
    validate_password(&body.password).map_err(AppError::BadRequest)?;
    svc.accept_invite(body.token, body.full_name, body.password).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn forgot_password(
    svc: web::Data<Arc<AuthService>>,
    body: web::Json<ForgotPasswordRequest>,
) -> Result<HttpResponse, AppError> {
    svc.forgot_password(&body.email).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}

pub async fn update_avatar(
    user: CurrentUser,
    svc: web::Data<Arc<AuthService>>,
    body: web::Json<UpdateAvatarRequest>,
) -> Result<HttpResponse, AppError> {
    svc.repo.update_avatar(user.id, &body.avatar_url).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(serde_json::json!({ "avatar_url": body.avatar_url }))))
}

pub async fn reset_user_password(
    user: CurrentUser,
    svc: web::Data<Arc<AuthService>>,
    path: web::Path<Uuid>,
    body: web::Json<ResetPasswordRequest>,
) -> Result<HttpResponse, AppError> {
    let target_id = path.into_inner();

    // Fetch the target user's profile to check their role
    let target_profile = svc.repo
        .find_profile_by_id(target_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    match user.role {
        Role::SuperAdmin => {
            // Can reset any user
        }
        Role::Hr => {
            // Can only reset drivers and owners
            if target_profile.role != Role::Driver && target_profile.role != Role::Owner {
                return Err(AppError::Forbidden("HR can only reset passwords for drivers and owners".into()));
            }
        }
        _ => {
            return Err(AppError::Forbidden("Insufficient permissions".into()));
        }
    }

    validate_password(&body.password).map_err(AppError::BadRequest)?;

    svc.reset_user_password(user.id, &user.role, target_id, &body.password).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(())))
}
