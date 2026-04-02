pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/dashboard", web::get().to(handlers::dashboard))
       .service(
           web::scope("/reports")
               .route("/drivers", web::get().to(handlers::driver_summary))
               .route("/trips", web::get().to(handlers::trip_detail))
               .route("/finance", web::get().to(handlers::finance_summary)),
       );
}
