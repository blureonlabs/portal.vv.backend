pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/owners")
            .route("", web::get().to(handlers::list_owners))
            .route("", web::post().to(handlers::create_owner))
            .route("/create-with-account", web::post().to(handlers::create_owner_with_account))
            .route("/{id}", web::get().to(handlers::get_owner))
            .route("/{id}", web::put().to(handlers::update_owner))
    )
    .service(
        web::scope("/owner")
            .route("/me", web::get().to(handlers::owner_me))
    );
}
