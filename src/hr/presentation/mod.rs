pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/hr/requests")
            .route("", web::get().to(handlers::list_leave))
            .route("", web::post().to(handlers::submit_leave))
            .route("/{id}/approve", web::put().to(handlers::approve_leave))
            .route("/{id}/reject", web::put().to(handlers::reject_leave)),
    );
}
