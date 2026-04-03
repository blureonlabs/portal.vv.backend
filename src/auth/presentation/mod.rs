pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/me", web::get().to(handlers::me))
            .route("/me/avatar", web::put().to(handlers::update_avatar))
            .route("/accept-invite", web::post().to(handlers::accept_invite))
            .route("/forgot-password", web::post().to(handlers::forgot_password))
    )
    .service(
        web::scope("/users")
            .route("", web::get().to(handlers::list_users))
            .route("/invite", web::post().to(handlers::invite_user))
            .route("/invites", web::get().to(handlers::list_invites))
            .route("/invites/{id}/revoke", web::put().to(handlers::revoke_invite))
            .route("/invites/{id}/resend", web::post().to(handlers::resend_invite))
            .route("/{id}/password", web::put().to(handlers::reset_user_password))
    );
}
