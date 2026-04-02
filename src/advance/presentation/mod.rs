pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/advances")
            .route("", web::get().to(handlers::list_advances))
            .route("", web::post().to(handlers::request_advance))
            .route("/{id}/approve", web::put().to(handlers::approve_advance))
            .route("/{id}/reject", web::put().to(handlers::reject_advance))
            .route("/{id}/pay", web::put().to(handlers::pay_advance)),
    );
}
