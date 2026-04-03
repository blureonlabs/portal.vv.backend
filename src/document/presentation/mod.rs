pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/documents")
            .route("", web::get().to(handlers::list_documents))
            .route("", web::post().to(handlers::create_document))
            .route("/{id}", web::delete().to(handlers::delete_document)),
    );
}
