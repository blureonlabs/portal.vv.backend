pub mod handlers;
pub mod dto;

use actix_web::web;
use handlers::*;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/config")
            .route("/expense-categories", web::get().to(list_expense_categories))
            .route("/expense-categories", web::post().to(create_expense_category))
            .route("/expense-categories/{id}", web::put().to(update_expense_category))
            .route("/expense-categories/{id}", web::delete().to(delete_expense_category))
            .route("/leave-types", web::get().to(list_leave_types))
            .route("/leave-types", web::post().to(create_leave_type))
            .route("/leave-types/{id}", web::put().to(update_leave_type))
            .route("/leave-types/{id}", web::delete().to(delete_leave_type))
            .route("/document-types", web::get().to(list_document_types))
            .route("/document-types", web::post().to(create_document_type))
            .route("/document-types/{id}", web::put().to(update_document_type))
            .route("/document-types/{id}", web::delete().to(delete_document_type))
    );
}
