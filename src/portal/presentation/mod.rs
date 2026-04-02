pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/me")
            .route("", web::get().to(handlers::get_me))
            .route("/earnings", web::get().to(handlers::get_earnings)),
    );
}
