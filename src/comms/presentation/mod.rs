pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/comms")
            .route("/broadcasts", web::get().to(handlers::list_broadcasts))
            .route("/broadcast", web::post().to(handlers::send_broadcast))
    );
}
