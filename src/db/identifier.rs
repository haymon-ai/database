//! Database and table name identifier validation.
//!
//! Ensures identifiers are safe for use in SQL by restricting to
//! alphanumeric characters and underscores.

use crate::error::AppError;

/// Validates that `name` is a safe SQL identifier.
///
/// # Errors
///
/// Returns [`AppError::InvalidIdentifier`] if the name is empty,
/// starts with a digit, or contains characters other than
/// alphanumeric and underscore.
pub fn validate_identifier(name: &str) -> Result<(), AppError> {
    if name.is_empty() {
        return Err(AppError::InvalidIdentifier(name.to_string()));
    }
    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return Err(AppError::InvalidIdentifier(name.to_string()));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(AppError::InvalidIdentifier(name.to_string()));
    }
    Ok(())
}

/// Wraps `name` in backticks for safe use in SQL DDL.
pub fn backtick_escape(name: &str) -> String {
    format!("`{name}`")
}
