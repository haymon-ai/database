//! `IP_ADDRESS` recognizer (IPv4 + IPv6) with parse-validation.
//!
//! Patterns ported from Presidio's `IpRecognizer`. Lookbehind/lookahead boundary
//! anchors require `fancy-regex`.

use crate::pattern::Pattern;
use crate::recognizer::{IpAddressValidator, PatternRecognizer, entity};
use crate::score::Score;

/// Build the `IP_ADDRESS` recognizer.
///
/// # Panics
///
/// Panics only if any bundled regex source or score constant is rejected at construction.
#[must_use]
pub fn ip_address() -> PatternRecognizer {
    let s06 = Score::new(0.6).expect("0.6 in range");

    let ipv4 = Pattern::new_fancy(
        "IPv4",
        r"\b(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(?:/(?:[0-2]?\d|3[0-2]))?\b",
        s06,
    )
    .expect("static IPv4 pattern compiles");

    let ipv6 = Pattern::new_fancy(
        "IPv6",
        r"(?<![\w:])(?:(?:[0-9A-Fa-f]{1,4}:){7}[0-9A-Fa-f]{1,4}|(?:[0-9A-Fa-f]{1,4}:){1,7}:|:(?::[0-9A-Fa-f]{1,4}){1,7}|(?:[0-9A-Fa-f]{1,4}:){1,6}:[0-9A-Fa-f]{1,4}|(?:[0-9A-Fa-f]{1,4}:){1,5}(?::[0-9A-Fa-f]{1,4}){1,2}|(?:[0-9A-Fa-f]{1,4}:){1,4}(?::[0-9A-Fa-f]{1,4}){1,3}|(?:[0-9A-Fa-f]{1,4}:){1,3}(?::[0-9A-Fa-f]{1,4}){1,4}|(?:[0-9A-Fa-f]{1,4}:){1,2}(?::[0-9A-Fa-f]{1,4}){1,5}|[0-9A-Fa-f]{1,4}:(?::[0-9A-Fa-f]{1,4}){1,6}|:(?::[0-9A-Fa-f]{1,4}){1,6})(?:/(?:12[0-8]|1[01]\d|[1-9]?\d))?(?![\w:])",
        s06,
    )
    .expect("static IPv6 pattern compiles");

    PatternRecognizer::new(entity::IP_ADDRESS, vec![ipv4, ipv6])
        .expect("non-empty pattern list")
        .with_name("IpRecognizer")
        .with_validator(IpAddressValidator)
}
