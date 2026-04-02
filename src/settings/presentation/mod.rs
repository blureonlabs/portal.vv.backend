pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .route("", web::get().to(handlers::list_settings))
            .route("/{key}", web::put().to(handlers::update_setting)),
    );
}
