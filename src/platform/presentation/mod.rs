pub mod handlers;

use actix_web::web;
use handlers::list_platforms;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/platforms", web::get().to(list_platforms));
}
