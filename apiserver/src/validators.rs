//! Shared validation functions for request inputs.
//!
//! Extracted from handlers to keep them thin and focused on request/response glue.

use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;

const MIN_PASSWORD_LEN: usize = 8;
const MAX_PASSWORD_LEN: usize = 128;

/// Validate password strength: length (8–128) and at least one digit or special char.
pub fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(AppError::validation(I18nKey::PasswordTooShort));
    }
    if password.len() > MAX_PASSWORD_LEN {
        return Err(AppError::validation(I18nKey::PasswordTooLong));
    }

    if !password
        .chars()
        .any(|c| c.is_ascii_digit() || c.is_ascii_punctuation())
    {
        return Err(AppError::validation(I18nKey::PasswordComplexity));
    }

    Ok(())
}

/// Validate email format: local@domain.tld with structural checks.
pub fn validate_email(email: &str) -> AppResult<()> {
    let trimmed = email.trim();
    if trimmed.is_empty() || trimmed.len() > 254 {
        return Err(AppError::validation(I18nKey::InvalidEmailFormat));
    }

    let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
    if parts.len() != 2 {
        return Err(AppError::validation(I18nKey::InvalidEmailFormat));
    }

    let (local, domain) = (parts[0], parts[1]);

    if local.is_empty() || local.len() > 64 {
        return Err(AppError::validation(I18nKey::InvalidEmailFormat));
    }

    if domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
    {
        return Err(AppError::validation(I18nKey::InvalidEmailFormat));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_ok() {
        assert!(validate_password("secret123!").is_ok());
        assert!(validate_password("MyP@ssw0rd").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        assert!(validate_password("Abc1!").is_err());
    }

    #[test]
    fn test_validate_password_no_digit_or_special() {
        assert!(validate_password("abcdefghijklmn").is_err());
    }

    #[test]
    fn test_validate_email_ok() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("a@b.co").is_ok());
    }

    #[test]
    fn test_validate_email_no_at() {
        assert!(validate_email("userexample.com").is_err());
    }

    #[test]
    fn test_validate_email_empty() {
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_validate_email_double_dot() {
        assert!(validate_email("user@exa..mple.com").is_err());
    }
}
