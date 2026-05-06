//! PEM private-key block type validator.

use crate::recognizer::{ValidationOutcome, Validator};

/// PEM private-key block type validator: BEGIN-type MUST equal END-type.
#[derive(Debug, Default, Clone, Copy)]
pub struct PrivateKeyTypeValidator;

impl Validator for PrivateKeyTypeValidator {
    fn validate(&self, candidate: &str) -> ValidationOutcome {
        let Some(begin_label) = candidate
            .split_once("-----BEGIN ")
            .and_then(|(_, rest)| rest.split_once("-----"))
            .map(|(label, _)| label.trim())
        else {
            return ValidationOutcome::Invalid;
        };
        let Some(end_label) = candidate
            .rsplit_once("-----END ")
            .and_then(|(_, rest)| rest.split_once("-----"))
            .map(|(label, _)| label.trim())
        else {
            return ValidationOutcome::Invalid;
        };
        ValidationOutcome::from_bool(begin_label == end_label && begin_label.contains("PRIVATE KEY"))
    }
}
