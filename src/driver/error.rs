use crate::common::error::AppError;

/// Domain errors specific to the driver feature.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Driver not found: {0}")]
    NotFound(String),

    #[error("Driver profile already exists: {0}")]
    AlreadyExists(String),

    #[error("{0}")]
    InvalidState(String),

    #[error("Driver has an active vehicle assignment")]
    HasActiveVehicle,

    #[error("Driver account is deactivated")]
    Deactivated,
}

impl From<DriverError> for AppError {
    fn from(e: DriverError) -> AppError {
        match e {
            DriverError::NotFound(msg) => AppError::NotFound(msg),
            DriverError::AlreadyExists(msg) => AppError::Conflict(msg),
            DriverError::InvalidState(msg) => AppError::BadRequest(msg),
            DriverError::HasActiveVehicle => {
                AppError::BadRequest("Driver has an active vehicle assignment".into())
            }
            DriverError::Deactivated => {
                AppError::Forbidden("Driver account is deactivated".into())
            }
        }
    }
}
