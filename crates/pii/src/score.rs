//! Confidence score newtype wrapping `f32` constrained to `[0.0, 1.0]`.

use crate::error::PatternError;

/// Highest score the engine emits; assigned by a passing validator (FR-004).
pub const MAX_SCORE: Score = Score(1.0);

/// Lowest score the engine recognises; results at this score never surface.
pub const MIN_SCORE: Score = Score(0.0);

/// Confidence score in `[0.0, 1.0]`; non-finite or out-of-range values are rejected at construction.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Score(f32);

impl Score {
    /// Construct a score, returning `Err` for non-finite or out-of-range values.
    ///
    /// # Errors
    ///
    /// Returns [`PatternError::InvalidScore`] when `value` is not finite or falls outside
    /// the inclusive `[0.0, 1.0]` interval.
    pub fn new(value: f32) -> Result<Self, PatternError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(PatternError::InvalidScore { value })
        }
    }

    /// Return the underlying `f32` in `[0.0, 1.0]`.
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_SCORE, MIN_SCORE, Score};
    use crate::error::PatternError;

    #[test]
    fn rejects_nan() {
        let err = Score::new(f32::NAN).unwrap_err();
        assert!(matches!(err, PatternError::InvalidScore { value } if value.is_nan()));
    }

    #[test]
    fn rejects_above_one() {
        let err = Score::new(1.0001).unwrap_err();
        assert!(matches!(err, PatternError::InvalidScore { .. }));
    }

    #[test]
    fn rejects_below_zero() {
        let err = Score::new(-0.0001).unwrap_err();
        assert!(matches!(err, PatternError::InvalidScore { .. }));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn accepts_endpoints() {
        // Exact bit-pattern check is intentional: Score::new must round-trip the input.
        assert_eq!(Score::new(0.0).unwrap().as_f32(), 0.0);
        assert_eq!(Score::new(1.0).unwrap().as_f32(), 1.0);
    }

    #[test]
    fn ordering() {
        let lo = Score::new(0.1).unwrap();
        let hi = Score::new(0.9).unwrap();
        assert!(lo < hi);
        assert!(MIN_SCORE < MAX_SCORE);
    }
}
