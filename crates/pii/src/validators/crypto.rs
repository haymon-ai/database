//! Bitcoin address checksum validator: `Base58Check` (P2PKH/P2SH) and Bech32/Bech32m (segwit).

use sha2::{Digest, Sha256};

use crate::ValidationOutcome;

const BASE58_ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const BECH32_CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const BECH32_CONST: u32 = 1;
const BECH32M_CONST: u32 = 0x2BC8_30A3;
const BECH32_GEN: [u32; 5] = [0x3B6A_57B2, 0x2650_8E6D, 0x1EA1_19FA, 0x3D42_33DD, 0x2A14_62B3];

/// Validate a Bitcoin address candidate by checksum.
///
/// `1`/`3` prefix → `Base58Check`; `bc1` prefix → Bech32/Bech32m; anything else
/// (including ETH `0x...`) returns [`ValidationOutcome::Unknown`].
pub(super) fn validate(candidate: &str) -> ValidationOutcome {
    let bytes = candidate.as_bytes();
    if bytes.first().is_some_and(|&b| b == b'1' || b == b'3') {
        return ValidationOutcome::from_bool(base58check_passes(candidate));
    }
    if candidate.starts_with("bc1") {
        return ValidationOutcome::from_bool(bech32_passes(candidate));
    }
    ValidationOutcome::Unknown
}

/// Verify a 25-byte `Base58Check` payload via double-SHA-256 over its first 21 bytes.
fn base58check_passes(candidate: &str) -> bool {
    let bytes = candidate.as_bytes();
    let orig_len = bytes.len();
    if !(25..=35).contains(&orig_len) {
        return false;
    }

    let leading_ones = bytes.iter().take_while(|&&b| b == b'1').count();
    let trimmed = &bytes[leading_ones..];

    let mut out = [0u8; 25];
    for &ch in trimmed {
        let Some(idx) = BASE58_ALPHABET.iter().position(|&c| c == ch) else {
            return false;
        };
        let mut carry = u32::try_from(idx).expect("Base58 index < 58");
        for byte in out.iter_mut().rev() {
            carry += u32::from(*byte) * 58;
            *byte = (carry & 0xFF) as u8;
            carry >>= 8;
        }
        if carry != 0 {
            return false;
        }
    }

    if leading_ones > 25 || out[..leading_ones].iter().any(|&b| b != 0) {
        return false;
    }
    let expected_version = match bytes.first() {
        Some(b'1') => 0x00,
        Some(b'3') => 0x05,
        _ => return false,
    };
    if out[0] != expected_version {
        return false;
    }

    let h1 = Sha256::digest(&out[..21]);
    let h2 = Sha256::digest(h1);
    h2[..4] == out[21..25]
}

/// Verify a Bech32 / Bech32m address per BIP-173 / BIP-350.
fn bech32_passes(candidate: &str) -> bool {
    let raw = candidate.as_bytes();
    if raw.len() > 90 {
        return false;
    }
    let mut has_lower = false;
    let mut has_upper = false;
    for &b in raw {
        if !(33..=126).contains(&b) {
            return false;
        }
        if b.is_ascii_lowercase() {
            has_lower = true;
        }
        if b.is_ascii_uppercase() {
            has_upper = true;
        }
    }
    if has_lower && has_upper {
        return false;
    }

    let mut lower = [0u8; 90];
    for (i, &b) in raw.iter().enumerate() {
        lower[i] = b.to_ascii_lowercase();
    }
    let bech = &lower[..raw.len()];

    let Some(pos) = bech.iter().rposition(|&b| b == b'1') else {
        return false;
    };
    if pos < 1 || pos + 7 > bech.len() {
        return false;
    }
    let hrp = &bech[..pos];
    let data = &bech[pos + 1..];

    let mut data_values = [0u8; 90];
    for (i, &b) in data.iter().enumerate() {
        let Some(idx) = BECH32_CHARSET.iter().position(|&c| c == b) else {
            return false;
        };
        data_values[i] = u8::try_from(idx).expect("Bech32 charset index < 32");
    }

    let mut polymod_input = [0u8; 90 + 90 + 1];
    let mut len = 0usize;
    for &b in hrp {
        polymod_input[len] = b >> 5;
        len += 1;
    }
    polymod_input[len] = 0;
    len += 1;
    for &b in hrp {
        polymod_input[len] = b & 31;
        len += 1;
    }
    for v in &data_values[..data.len()] {
        polymod_input[len] = *v;
        len += 1;
    }

    let chk = bech32_polymod(&polymod_input[..len]);
    chk == BECH32_CONST || chk == BECH32M_CONST
}

/// Compute the Bech32 polymod checksum over a sequence of 5-bit values.
fn bech32_polymod(values: &[u8]) -> u32 {
    let mut chk: u32 = 1;
    for &v in values {
        let top = chk >> 25;
        chk = ((chk & 0x01FF_FFFF) << 5) ^ u32::from(v);
        for (i, g) in BECH32_GEN.iter().enumerate() {
            if (top >> i) & 1 == 1 {
                chk ^= *g;
            }
        }
    }
    chk
}

#[cfg(test)]
mod tests {
    use super::validate;
    use crate::ValidationOutcome;

    #[test]
    fn base58check_p2pkh_valid() {
        assert_eq!(validate("16Yeky6GMjeNkAiNcBY7ZhrLoMSgg1BoyZ"), ValidationOutcome::Valid);
    }

    #[test]
    fn base58check_p2sh_valid() {
        assert_eq!(validate("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"), ValidationOutcome::Valid);
    }

    #[test]
    fn base58check_invalid_checksum() {
        assert_eq!(
            validate("16Yeky6GMjeNkAiNcBY7ZhrLoMSgg1BoyZ2"),
            ValidationOutcome::Invalid
        );
    }

    #[test]
    fn base58check_rejects_non_alphabet_char() {
        // `0` is in the recognizer regex char class but not in Base58.
        assert_eq!(
            validate("10Yeky6GMjeNkAiNcBY7ZhrLoMSgg1BoyZ"),
            ValidationOutcome::Invalid
        );
    }

    #[test]
    fn bech32_valid() {
        assert_eq!(
            validate("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
            ValidationOutcome::Valid
        );
    }

    #[test]
    fn bech32m_valid() {
        assert_eq!(
            validate("bc1p5d7rjq7g6rdk2yhzks9smlaqtedr4dekq08ge8ztwac72sfr9rusxg3297"),
            ValidationOutcome::Valid
        );
    }

    #[test]
    fn bech32_rejects_mixed_case() {
        assert_eq!(
            validate("bc1Qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
            ValidationOutcome::Invalid
        );
    }

    #[test]
    fn bech32_rejects_charset_violation() {
        // `b` is not in BECH32_CHARSET.
        assert_eq!(
            validate("bc1bar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
            ValidationOutcome::Invalid
        );
    }

    #[test]
    fn eth_address_abstains() {
        assert_eq!(
            validate("0x52908400098527886E0F7030069857D2E4169EE7"),
            ValidationOutcome::Unknown
        );
    }
}
