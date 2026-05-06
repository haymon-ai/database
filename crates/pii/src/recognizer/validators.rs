//! Built-in validators: `Luhn`, `IBAN` mod-97, IP-address parse-validation, plus
//! the `AndValidator` combinator.

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

/// Returns true iff the right-to-left Luhn weighted sum over `digits` is divisible by 10.
fn luhn_passes<I: IntoIterator<Item = u32>>(digits: I) -> bool
where
    I::IntoIter: DoubleEndedIterator,
{
    let sum: u32 = digits
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, d)| {
            if i.is_multiple_of(2) {
                d
            } else {
                let n = d * 2;
                if n > 9 { n - 9 } else { n }
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

/// Luhn checksum validator for credit-card numbers.
///
/// Strips spaces and dashes before checking, matching Presidio's
/// `replacement_pairs = [("-", ""), (" ", "")]`.
#[derive(Debug, Default, Clone, Copy)]
pub struct LuhnValidator;

impl Validator for LuhnValidator {
    fn validate(&self, candidate: &str) -> ValidationOutcome {
        // Buffer fits the longest valid card (19 digits); avoids a heap allocation.
        // Iterates bytes — credit-card candidates are ASCII-only after the regex match,
        // so the `chars()` decode is unnecessary work.
        let mut digits = [0u32; 19];
        let mut len = 0usize;
        for &b in candidate.as_bytes() {
            if !b.is_ascii_digit() {
                continue;
            }
            if len == digits.len() {
                return ValidationOutcome::Invalid;
            }
            digits[len] = u32::from(b - b'0');
            len += 1;
        }
        ValidationOutcome::from_bool((12..=19).contains(&len) && luhn_passes(digits[..len].iter().copied()))
    }
}

/// IBAN mod-97 validator. Accepts upper-case input; whitespace stripped before checking.
#[derive(Debug, Default, Clone, Copy)]
pub struct IbanValidator;

impl Validator for IbanValidator {
    fn validate(&self, candidate: &str) -> ValidationOutcome {
        // Longest legal IBAN is 34 chars; stack-buffer the cleaned form.
        let mut buf = [0u8; 34];
        let mut len = 0usize;
        for &b in candidate.as_bytes() {
            if b.is_ascii_whitespace() {
                continue;
            }
            if !b.is_ascii() || len == buf.len() {
                return ValidationOutcome::Invalid;
            }
            buf[len] = b.to_ascii_uppercase();
            len += 1;
        }
        if len < 15 {
            return ValidationOutcome::Invalid;
        }
        // Rearranged = tail (positions 4..len) followed by head (positions 0..4).
        let rearranged = buf[4..len].iter().chain(buf[..4].iter()).copied();
        ValidationOutcome::from_bool(mod97(rearranged) == Some(1))
    }
}

fn mod97<I: Iterator<Item = u8>>(bytes: I) -> Option<u32> {
    let mut remainder: u32 = 0;
    for b in bytes {
        if b.is_ascii_digit() {
            remainder = (remainder * 10 + u32::from(b - b'0')) % 97;
        } else if b.is_ascii_uppercase() {
            remainder = (remainder * 100 + u32::from(b - b'A' + 10)) % 97;
        } else {
            return None;
        }
    }
    Some(remainder)
}

/// Collect exactly `N` ASCII digits from `candidate`; returns `None` for any other count.
///
/// Iterates bytes (not chars) since every candidate that reaches a numeric
/// validator is ASCII-only post-regex-match.
fn collect_digits<const N: usize>(candidate: &str) -> Option<[u32; N]> {
    let mut out = [0u32; N];
    let mut i = 0usize;
    for &b in candidate.as_bytes() {
        if !b.is_ascii_digit() {
            continue;
        }
        if i == N {
            return None;
        }
        out[i] = u32::from(b - b'0');
        i += 1;
    }
    (i == N).then_some(out)
}

/// US Social Security Number validator. Rejects reserved area / group / serial values
/// — replaces the negative-lookahead constructs Presidio's regex used.
#[derive(Debug, Default, Clone, Copy)]
pub struct UsSsnValidator;

impl Validator for UsSsnValidator {
    fn validate(&self, candidate: &str) -> ValidationOutcome {
        let Some(digits) = collect_digits::<9>(candidate) else {
            return ValidationOutcome::Invalid;
        };
        let area = digits[0] * 100 + digits[1] * 10 + digits[2];
        let group = digits[3] * 10 + digits[4];
        let serial = digits[5] * 1000 + digits[6] * 100 + digits[7] * 10 + digits[8];
        let valid = area != 0 && area != 666 && area < 900 && group != 0 && serial != 0;
        ValidationOutcome::from_bool(valid)
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
        // Strip CIDR suffix `/N` and IPv6 zone identifier `%zone` in one split.
        let trimmed = candidate.split(['/', '%']).next().unwrap_or("");
        ValidationOutcome::from_bool(IpAddr::from_str(trimmed).is_ok())
    }
}

/// Combinator returning [`ValidationOutcome::Valid`] only if both children agree.
///
/// Truth table:
/// - Both `Valid` → `Valid`
/// - Either `Invalid` → `Invalid` (short-circuits)
/// - Otherwise → `Unknown`
#[derive(Debug, Clone, Copy)]
pub struct AndValidator<L, R> {
    /// Left-hand operand (evaluated first).
    pub left: L,
    /// Right-hand operand.
    pub right: R,
}

impl<L, R> AndValidator<L, R> {
    /// Compose two validators with AND semantics.
    pub const fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<L, R> Validator for AndValidator<L, R>
where
    L: Validator,
    R: Validator,
{
    fn validate(&self, candidate: &str) -> ValidationOutcome {
        match (self.left.validate(candidate), self.right.validate(candidate)) {
            (ValidationOutcome::Invalid, _) | (_, ValidationOutcome::Invalid) => ValidationOutcome::Invalid,
            (ValidationOutcome::Valid, ValidationOutcome::Valid) => ValidationOutcome::Valid,
            _ => ValidationOutcome::Unknown,
        }
    }

    fn validate_with_context(
        &self,
        candidate: &str,
        full_text: &str,
        span: std::ops::Range<usize>,
    ) -> ValidationOutcome {
        let l = self.left.validate_with_context(candidate, full_text, span.clone());
        if matches!(l, ValidationOutcome::Invalid) {
            return ValidationOutcome::Invalid;
        }
        let r = self.right.validate_with_context(candidate, full_text, span);
        match (l, r) {
            (_, ValidationOutcome::Invalid) => ValidationOutcome::Invalid,
            (ValidationOutcome::Valid, ValidationOutcome::Valid) => ValidationOutcome::Valid,
            _ => ValidationOutcome::Unknown,
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
