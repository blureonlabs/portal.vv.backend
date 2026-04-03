pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/trips")
            .route("", web::get().to(handlers::list_trips))
            .route("", web::post().to(handlers::create_trip))
            .route("/csv/template", web::get().to(handlers::csv_template))
            .route("/csv/preview", web::post().to(handlers::csv_preview))
            .route("/csv/import", web::post().to(handlers::csv_import))
            .route("/{id}", web::delete().to(handlers::delete_trip))
    );
}
