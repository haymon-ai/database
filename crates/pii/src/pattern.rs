//! Named regex with confidence score; lazily compiled and cached.

use std::fmt;
use std::sync::OnceLock;

use crate::error::PatternError;
use crate::score::Score;

/// Which regex engine compiles and runs the pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatternKind {
    /// `regex` crate; RE2-style, linear time, no lookaround.
    Regex,
    /// `fancy-regex` crate; supports lookbehind and lookahead.
    Fancy,
}

/// Compiled form of a [`Pattern`]; populated on first use.
pub(crate) enum CompiledPattern {
    Regex(regex::Regex),
    Fancy(fancy_regex::Regex),
}

impl fmt::Debug for CompiledPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Regex(_) => f.write_str("CompiledPattern::Regex(..)"),
            Self::Fancy(_) => f.write_str("CompiledPattern::Fancy(..)"),
        }
    }
}

/// Named regex pattern with a base confidence score.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pattern {
    name: String,
    regex: String,
    score: Score,
    kind: PatternKind,
    #[cfg_attr(feature = "serde", serde(skip))]
    compiled: OnceLock<CompiledPattern>,
}

impl Clone for Pattern {
    fn clone(&self) -> Self {
        // Compiled cache is intentionally dropped on clone; recompiles on first use.
        Self {
            name: self.name.clone(),
            regex: self.regex.clone(),
            score: self.score,
            kind: self.kind,
            compiled: OnceLock::new(),
        }
    }
}

impl Pattern {
    /// Build a [`PatternKind::Regex`] pattern; compiles eagerly to surface errors at construction.
    ///
    /// # Errors
    ///
    /// Returns [`PatternError::InvalidRegex`] when the source fails to compile under the
    /// `regex` crate. (Score validation happens via [`Score`]'s constructor before this call.)
    pub fn new(name: impl Into<String>, regex_src: impl Into<String>, score: Score) -> Result<Self, PatternError> {
        let regex_src = regex_src.into();
        let compiled = regex::Regex::new(&regex_src).map_err(|e| PatternError::InvalidRegex { source: Box::new(e) })?;
        let cell = OnceLock::new();
        let _ = cell.set(CompiledPattern::Regex(compiled));
        Ok(Self {
            name: name.into(),
            regex: regex_src,
            score,
            kind: PatternKind::Regex,
            compiled: cell,
        })
    }

    /// Build a [`PatternKind::Fancy`] pattern (lookbehind/lookahead-capable).
    ///
    /// # Errors
    ///
    /// Returns [`PatternError::InvalidRegex`] when the source fails to compile under
    /// `fancy-regex`.
    pub fn new_fancy(
        name: impl Into<String>,
        regex_src: impl Into<String>,
        score: Score,
    ) -> Result<Self, PatternError> {
        let regex_src = regex_src.into();
        let compiled =
            fancy_regex::Regex::new(&regex_src).map_err(|e| PatternError::InvalidRegex { source: Box::new(e) })?;
        let cell = OnceLock::new();
        let _ = cell.set(CompiledPattern::Fancy(compiled));
        Ok(Self {
            name: name.into(),
            regex: regex_src,
            score,
            kind: PatternKind::Fancy,
            compiled: cell,
        })
    }

    /// Pattern's human-readable name; surfaced in [`crate::AnalysisExplanation`].
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Regex source (the string the pattern was constructed with).
    #[must_use]
    pub fn regex(&self) -> &str {
        &self.regex
    }

    /// Base confidence score, before any validator promotion.
    #[must_use]
    pub fn score(&self) -> Score {
        self.score
    }

    /// Engine variant used to compile and run this pattern.
    #[must_use]
    pub fn kind(&self) -> PatternKind {
        self.kind
    }

    /// Access the compiled form; returns `Err` only on a re-compile race after a `serde`
    /// deserialise that left the cache empty.
    pub(crate) fn compiled(&self) -> Result<&CompiledPattern, PatternError> {
        if let Some(c) = self.compiled.get() {
            return Ok(c);
        }
        let new = match self.kind {
            PatternKind::Regex => CompiledPattern::Regex(
                regex::Regex::new(&self.regex).map_err(|e| PatternError::InvalidRegex { source: Box::new(e) })?,
            ),
            PatternKind::Fancy => CompiledPattern::Fancy(
                fancy_regex::Regex::new(&self.regex).map_err(|e| PatternError::InvalidRegex { source: Box::new(e) })?,
            ),
        };
        let _ = self.compiled.set(new);
        Ok(self.compiled.get().expect("cache was just populated"))
    }
}

#[cfg(test)]
mod tests {
    use super::{Pattern, PatternKind};
    use crate::error::PatternError;
    use crate::score::Score;

    fn s(v: f32) -> Score {
        Score::new(v).expect("valid score")
    }

    #[test]
    fn rejects_invalid_regex() {
        let err = Pattern::new("bad", "(unclosed", s(0.5)).unwrap_err();
        assert!(matches!(err, PatternError::InvalidRegex { .. }));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn accepts_valid_regex() {
        let p = Pattern::new("digits", r"\b\d+\b", s(0.5)).unwrap();
        assert_eq!(p.kind(), PatternKind::Regex);
        assert_eq!(p.score().as_f32(), 0.5);
        assert!(p.compiled().is_ok());
    }

    #[test]
    fn fancy_accepts_lookbehind() {
        let p = Pattern::new_fancy("ip_pre", r"(?<![\w:])\d+", s(0.6)).unwrap();
        assert_eq!(p.kind(), PatternKind::Fancy);
        assert!(p.compiled().is_ok());
    }

    #[test]
    fn regex_rejects_lookbehind() {
        let err = Pattern::new("bad_lb", r"(?<!a)b", s(0.5)).unwrap_err();
        assert!(matches!(err, PatternError::InvalidRegex { .. }));
    }
}
