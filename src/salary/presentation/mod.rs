pub mod dto;
pub mod handlers;

use actix_web::web;

use handlers::{generate_salary, get_salary, list_salaries};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/salaries/generate", web::post().to(generate_salary))
       .route("/salaries", web::get().to(list_salaries))
       .route("/salaries/{id}", web::get().to(get_salary));
}
