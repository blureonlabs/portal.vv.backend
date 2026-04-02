pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/drivers")
            .route("", web::get().to(handlers::list_drivers))
            .route("", web::post().to(handlers::create_driver))
            .route("/{id}", web::get().to(handlers::get_driver))
            .route("/{id}", web::put().to(handlers::update_driver))
            .route("/{id}/deactivate", web::put().to(handlers::deactivate_driver))
            .route("/{id}/activate", web::put().to(handlers::activate_driver))
            .route("/{id}/edits", web::get().to(handlers::list_driver_edits))
    );
}
