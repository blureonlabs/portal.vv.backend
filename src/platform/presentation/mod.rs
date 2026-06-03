pub mod handlers;
pub mod dto;

use actix_web::web;
use handlers::*;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/platforms", web::get().to(list_platforms))
       .route("/platforms", web::post().to(create_platform))
       .route("/platforms/{id}", web::put().to(update_platform))
       .route("/platforms/{id}", web::delete().to(deactivate_platform));
}
