pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/vehicles")
            .route("", web::get().to(handlers::list_vehicles))
            .route("", web::post().to(handlers::create_vehicle))
            .route("/{id}", web::get().to(handlers::get_vehicle))
            .route("/{id}", web::put().to(handlers::update_vehicle))
            .route("/{id}/assign", web::post().to(handlers::assign_driver))
            .route("/{id}/unassign", web::post().to(handlers::unassign_driver))
            .route("/{id}/assignments", web::get().to(handlers::list_assignments))
            .route("/{id}/service", web::get().to(handlers::list_service_records))
            .route("/{id}/service", web::post().to(handlers::add_service_record))
    );
}
