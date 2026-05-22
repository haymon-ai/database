//! International phone-number (E.164) length validator.
//!
//! Accepts a candidate only when it carries an explicit international
//! indicator — a leading `+` or the IDD prefix `00` — and its country code
//! plus national number total 8–15 significant digits (the E.164 bound). The
//! `+`/`00` gate is what keeps bare digit runs (timestamps, IDs, references)
//! out of the `PHONE_NUMBER` entity.

use crate::ValidationOutcome;

/// Validate an international phone-number candidate by E.164 length.
///
/// Returns [`ValidationOutcome::Unknown`] for accept (so the recognizer's
/// score stays at `0.4` and does not outrank higher-scored entities), and
/// [`ValidationOutcome::Invalid`] for reject.
pub(super) fn phone(candidate: &str) -> ValidationOutcome {
    let trimmed = candidate.trim_start();
    let had_plus = trimmed.starts_with('+');
    let had_idd = trimmed.starts_with("00");
    if !had_plus && !had_idd {
        return ValidationOutcome::Invalid;
    }

    let mut digits = [0u8; 17];
    let mut len = 0usize;
    for &b in candidate.as_bytes() {
        if b.is_ascii_digit() {
            if len == digits.len() {
                return ValidationOutcome::Invalid;
            }
            digits[len] = b - b'0';
            len += 1;
        }
    }

    let significant = if had_idd { digits.get(2..len) } else { digits.get(..len) };
    let Some(significant) = significant else {
        return ValidationOutcome::Invalid;
    };

    let accept = (8..=15).contains(&significant.len()) && significant.first().is_some_and(|&d| (1..=9).contains(&d));

    if accept {
        ValidationOutcome::Unknown
    } else {
        ValidationOutcome::Invalid
    }
}

#[cfg(test)]
mod tests {
    use super::phone;
    use crate::ValidationOutcome;

    fn is_valid(s: &str) -> bool {
        phone(s) != ValidationOutcome::Invalid
    }

    #[test]
    fn accepts_intl_numbers() {
        for s in [
            "+14155552671",
            "+44 20 7946 0958",
            "+49 30 12345678",
            "+43 12345678",
            "+49 36878 620-23924",
            "+43 5574 6706 0000",
            "+49 711 501-20726",
            "0049/5235/3-00",
            "0039/011/9346211",
            "0033/1/34317000",
            "0043/5572/401045",
        ] {
            assert!(is_valid(s), "should accept {s:?}");
        }
    }

    #[test]
    fn rejects_non_phone() {
        for s in [
            "(415) 555-2671",
            "4155552671",
            "02012345678",
            "030 12345678",
            "202110",
            "900000000",
            "1234567890",
            "4111111111111111",
            "000-12-3456",
            "07-1234567",
            "046 454 287",
            "01234567",
            "0461234567",
            "+12",
            "+1234567",
            "0046123",
            "00501",
            "+1234567890123456",
            "+012345678",
            "00012345678",
        ] {
            assert!(!is_valid(s), "should reject {s:?}");
        }
    }
}
