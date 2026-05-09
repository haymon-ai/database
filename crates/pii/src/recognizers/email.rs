//! `EMAIL_ADDRESS` recognizer.
//!
//! Pattern adapted from Presidio's `EmailRecognizer.PATTERNS["Email (Medium)"]`.

use super::Recognizer;
use crate::regex::Regex;
use crate::score::Score;
use crate::types::{Category, Entity};

/// Build the `EMAIL_ADDRESS` recognizer.
///
/// # Panics
///
/// Panics only if the bundled regex source or score literal is rejected at construction;
/// both are unit-tested.
#[must_use]
pub fn email() -> Recognizer {
    let pattern = Regex::new(
        "Email (Medium)",
        r"\b[A-Za-z0-9!#$%&'*+\-/=?^_`{|}~]+(?:\.[A-Za-z0-9!#$%&'*+\-/=?^_`{|}~]+)*@[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?(?:\.[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)+\b",
        Score::from_static(0.5),
    )
    .expect("static email pattern compiles");
    Recognizer::new(Entity::EmailAddress, vec![pattern])
        .expect("non-empty pattern list")
        .with_name("EmailRecognizer")
        .with_category(Category::Personal)
}
