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
