use crate::common::error::AppError;

/// Domain errors specific to the auth feature.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("User already registered: {0}")]
    AlreadyRegistered(String),

    #[error("{0}")]
    InvalidInvite(String),

    #[error("Invite not found: {0}")]
    InviteNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),
}

impl From<AuthError> for AppError {
    fn from(e: AuthError) -> AppError {
        match e {
            AuthError::AlreadyRegistered(msg) => AppError::Conflict(msg),
            AuthError::InvalidInvite(msg) => AppError::BadRequest(msg),
            AuthError::InviteNotFound(msg) => AppError::NotFound(msg),
            AuthError::UserNotFound(msg) => AppError::NotFound(msg),
        }
    }
}
