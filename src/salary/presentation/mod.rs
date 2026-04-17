pub mod dto;
pub mod handlers;

use actix_web::web;

use handlers::{approve_salary, fetch_earnings, generate_salary, get_salary, list_salaries, mark_salary_paid};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/salaries/generate", web::post().to(generate_salary))
       .route("/salaries/earnings", web::get().to(fetch_earnings))
       .route("/salaries", web::get().to(list_salaries))
       .route("/salaries/{id}", web::get().to(get_salary))
       .route("/salaries/{id}/approve", web::post().to(approve_salary))
       .route("/salaries/{id}/pay", web::post().to(mark_salary_paid));
}
