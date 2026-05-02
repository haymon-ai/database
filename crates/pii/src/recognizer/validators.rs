//! Built-in validators: `Luhn`, `IBAN` mod-97, and IP-address parse-validation.

use std::net::IpAddr;
use std::str::FromStr;

use super::{ValidationOutcome, Validator};

/// Default validator that abstains on every input.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopValidator;

impl Validator for NoopValidator {
    fn validate(&self, _candidate: &str) -> ValidationOutcome {
        ValidationOutcome::Unknown
    }
}

/// Luhn checksum validator for credit-card numbers.
///
/// Strips spaces and dashes before checking, matching Presidio's
/// `replacement_pairs = [("-", ""), (" ", "")]`.
#[derive(Debug, Default, Clone, Copy)]
pub struct LuhnValidator;

impl LuhnValidator {
    fn luhn_ok(digits: &[u8]) -> bool {
        let mut sum: u32 = 0;
        let mut alt = false;
        for &d in digits.iter().rev() {
            let mut n = u32::from(d);
            if alt {
                n *= 2;
                if n > 9 {
                    n -= 9;
                }
            }
            sum += n;
            alt = !alt;
        }
        sum.is_multiple_of(10)
    }
}

impl Validator for LuhnValidator {
    fn validate(&self, candidate: &str) -> ValidationOutcome {
        let digits: Vec<u8> = candidate
            .chars()
            .filter(|c| !matches!(*c, '-' | ' '))
            .filter_map(|c| c.to_digit(10))
            .map(|d| u8::try_from(d).expect("base-10 digit fits in u8"))
            .collect();
        if !(12..=19).contains(&digits.len()) {
            return ValidationOutcome::Invalid;
        }
        if Self::luhn_ok(&digits) {
            ValidationOutcome::Valid
        } else {
            ValidationOutcome::Invalid
        }
    }
}

/// IBAN mod-97 validator. Accepts upper-case input; spaces stripped before checking.
#[derive(Debug, Default, Clone, Copy)]
pub struct IbanValidator;

impl IbanValidator {
    fn mod97(rearranged: &str) -> Option<u32> {
        let mut numeric = String::with_capacity(rearranged.len() * 2);
        for c in rearranged.chars() {
            if c.is_ascii_digit() {
                numeric.push(c);
            } else if c.is_ascii_uppercase() {
                let v = (c as u8) - b'A' + 10;
                numeric.push_str(&v.to_string());
            } else {
                return None;
            }
        }
        // Compute mod 97 in chunks to avoid overflow; chunk length is at most 7 digits.
        let mut remainder: u32 = 0;
        for chunk in numeric.as_bytes().chunks(7) {
            let s = std::str::from_utf8(chunk).ok()?;
            let n: u32 = s.parse().ok()?;
            let chunk_len = u32::try_from(s.len()).ok()?;
            remainder = (remainder * 10u32.pow(chunk_len) + n) % 97;
        }
        Some(remainder)
    }
}

impl Validator for IbanValidator {
    fn validate(&self, candidate: &str) -> ValidationOutcome {
        let cleaned: String = candidate
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| c.to_ascii_uppercase())
            .collect();
        if cleaned.len() < 15 || cleaned.len() > 34 {
            return ValidationOutcome::Invalid;
        }
        let (head, tail) = cleaned.split_at(4);
        let rearranged = format!("{tail}{head}");
        match Self::mod97(&rearranged) {
            Some(1) => ValidationOutcome::Valid,
            Some(_) | None => ValidationOutcome::Invalid,
        }
    }
}

/// IP-address validator that delegates to [`std::net::IpAddr::from_str`].
///
/// CIDR-like suffixes (`/24`, `/64`) are stripped before parsing; only the
/// address portion is parse-validated. A bare IPv6 zone identifier (`%eth0`)
/// is also stripped because `from_str` rejects it on stable today.
#[derive(Debug, Default, Clone, Copy)]
pub struct IpAddressValidator;

impl Validator for IpAddressValidator {
    fn validate(&self, candidate: &str) -> ValidationOutcome {
        let trimmed = candidate.split('/').next().unwrap_or(candidate);
        let trimmed = trimmed.split('%').next().unwrap_or(trimmed);
        if IpAddr::from_str(trimmed).is_ok() {
            ValidationOutcome::Valid
        } else {
            ValidationOutcome::Invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IbanValidator, IpAddressValidator, LuhnValidator, ValidationOutcome, Validator};

    #[test]
    fn luhn_valid_visa() {
        assert_eq!(LuhnValidator.validate("4111-1111-1111-1111"), ValidationOutcome::Valid);
    }

    #[test]
    fn luhn_invalid_visa() {
        assert_eq!(
            LuhnValidator.validate("4111-1111-1111-1112"),
            ValidationOutcome::Invalid
        );
    }

    #[test]
    fn luhn_rejects_short() {
        assert_eq!(LuhnValidator.validate("4111111"), ValidationOutcome::Invalid);
    }

    #[test]
    fn iban_valid_de() {
        // Wikipedia example
        assert_eq!(
            IbanValidator.validate("DE89 3704 0044 0532 0130 00"),
            ValidationOutcome::Valid
        );
    }

    #[test]
    fn iban_invalid_check_digits() {
        assert_eq!(
            IbanValidator.validate("DE00 3704 0044 0532 0130 00"),
            ValidationOutcome::Invalid
        );
    }

    #[test]
    fn ip_valid_v4() {
        assert_eq!(IpAddressValidator.validate("192.168.1.1"), ValidationOutcome::Valid);
    }

    #[test]
    fn ip_invalid_v4() {
        assert_eq!(IpAddressValidator.validate("192.168.1.999"), ValidationOutcome::Invalid);
    }

    #[test]
    fn ip_valid_v6() {
        assert_eq!(IpAddressValidator.validate("::1"), ValidationOutcome::Valid);
    }

    #[test]
    fn ip_with_cidr_suffix() {
        assert_eq!(IpAddressValidator.validate("10.0.0.0/24"), ValidationOutcome::Valid);
    }
}
