use actix_web::web;

use crate::portal::presentation::routes;

/// Portal has no services — just routes.
pub fn register(cfg: &mut web::ServiceConfig) {
    routes(cfg);
}
