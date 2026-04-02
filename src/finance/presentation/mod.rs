pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/finance")
            .route("/expenses", web::get().to(handlers::list_expenses))
            .route("/expenses", web::post().to(handlers::create_expense))
            .route("/handovers", web::get().to(handlers::list_handovers))
            .route("/handovers", web::post().to(handlers::create_handover))
    );
}
