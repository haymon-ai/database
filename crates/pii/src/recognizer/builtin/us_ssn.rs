//! `US_SSN` recognizer.
//!
//! Negative-lookahead groups exclude reserved area numbers (000, 666),
//! reserved group numbers (00), and reserved serial numbers (0000), so the
//! pattern uses `fancy-regex`.

use crate::pattern::Pattern;
use crate::recognizer::{PatternRecognizer, entity};
use crate::score::Score;

/// Build the `US_SSN` recognizer.
///
/// # Panics
///
/// Panics only if the bundled regex source or score constant is rejected at construction.
#[must_use]
pub fn us_ssn() -> PatternRecognizer {
    let pattern = Pattern::new_fancy(
        "US SSN",
        r"\b(?!000|666)[0-8]\d{2}[- ]?(?!00)\d{2}[- ]?(?!0000)\d{4}\b",
        Score::new(0.6).expect("0.6 in range"),
    )
    .expect("static SSN pattern compiles");
    PatternRecognizer::new(entity::US_SSN, vec![pattern])
        .expect("non-empty pattern list")
        .with_name("UsSsnRecognizer")
}
