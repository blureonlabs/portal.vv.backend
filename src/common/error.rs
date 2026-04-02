use actix_web::HttpResponse;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[allow(dead_code)]
    #[error("Unprocessable: {0}")]
    Unprocessable(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        use actix_web::http::StatusCode;

        let status = match self {
            AppError::NotFound(_)      => StatusCode::NOT_FOUND,
            AppError::Forbidden(_)     => StatusCode::FORBIDDEN,
            AppError::Unauthorized     => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_)    => StatusCode::BAD_REQUEST,
            AppError::Conflict(_)      => StatusCode::CONFLICT,
            AppError::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Database(_)      => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_)      => StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpResponse::build(status).json(json!({
            "data": null,
            "error": self.to_string(),
            "meta": null
        }))
    }
}
