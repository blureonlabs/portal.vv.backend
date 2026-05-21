use crate::common::error::AppError;

/// Domain errors specific to the salary feature.
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum SalaryError {
    #[error("Salary already exists for this period: {0}")]
    DuplicatePeriod(String),

    #[error("Only draft salaries can be edited")]
    NotDraft,
}

impl From<SalaryError> for AppError {
    fn from(e: SalaryError) -> AppError {
        match e {
            SalaryError::DuplicatePeriod(msg) => AppError::Conflict(msg),
            SalaryError::NotDraft => {
                AppError::BadRequest("Only draft salaries can be edited".into())
            }
        }
    }
}
