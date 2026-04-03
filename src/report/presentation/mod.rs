pub mod dto;
pub mod handlers;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/dashboard", web::get().to(handlers::dashboard))
       .route("/dashboard/driver-financials", web::get().to(handlers::driver_financials))
       .service(
           web::scope("/reports")
               .route("/drivers", web::get().to(handlers::driver_summary))
               .route("/trips", web::get().to(handlers::trip_detail))
               .route("/finance", web::get().to(handlers::finance_summary))
               .route("/advances", web::get().to(handlers::advance_report))
               .route("/cash-flow", web::get().to(handlers::cash_flow_report))
               .route("/leave", web::get().to(handlers::leave_report))
               .route("/salary", web::get().to(handlers::salary_report))
               .route("/vehicles", web::get().to(handlers::vehicle_report)),
       );
}
