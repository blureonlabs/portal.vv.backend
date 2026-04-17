use rust_decimal::Decimal;

pub fn validate_amount(field: &str, amount: Decimal) -> Result<(), super::error::AppError> {
    if amount < Decimal::ZERO {
        return Err(super::error::AppError::BadRequest(format!("{} cannot be negative", field)));
    }
    if amount > Decimal::from(1_000_000) {
        return Err(super::error::AppError::BadRequest(format!("{} exceeds maximum allowed value", field)));
    }
    Ok(())
}

pub fn validate_string_length(field: &str, value: &str, max_len: usize) -> Result<(), super::error::AppError> {
    if value.len() > max_len {
        return Err(super::error::AppError::BadRequest(format!("{} exceeds maximum length of {} characters", field, max_len)));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".into());
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain an uppercase letter".into());
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain a lowercase letter".into());
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain a number".into());
    }
    Ok(())
}
