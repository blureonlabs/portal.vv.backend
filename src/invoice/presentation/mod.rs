pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/invoices")
            .route("", web::get().to(handlers::list_invoices))
            .route("", web::post().to(handlers::generate_invoice))
            .route("/{id}", web::get().to(handlers::get_invoice)),
    );
}
