//! Deny-list helper that compiles literal terms into a single recognizer.

use crate::error::RecognizerError;
use crate::pattern::Pattern;
use crate::score::Score;

use super::{EntityType, PatternRecognizer};

/// Build a recognizer that matches whole-word occurrences of the supplied `terms`.
///
/// The compiled regex anchors each term with non-word boundaries (`(?:^|(?<=\W))` and
/// `(?:(?=\W)|$)`) so substrings of larger words do not match. Uses `fancy-regex` because
/// the leading lookbehind is non-fixed-width.
///
/// # Errors
///
/// Returns [`RecognizerError::EmptyPatternList`] when `terms` is empty. Propagates
/// [`RecognizerError::EmptyPatternList`] (re-mapped from a regex compile error) if the
/// produced pattern fails to compile, which indicates an internal bug — callers may treat
/// it as `unreachable` in practice.
pub fn deny_list_recognizer<S: AsRef<str>>(
    entity_type: EntityType,
    terms: &[S],
    score: Score,
) -> Result<PatternRecognizer, RecognizerError> {
    if terms.is_empty() {
        return Err(RecognizerError::EmptyPatternList);
    }
    let escaped: Vec<String> = terms.iter().map(|t| regex::escape(t.as_ref())).collect();
    let alternation = escaped.join("|");
    let regex_src = format!(r"(?:^|(?<=\W))(?:{alternation})(?:(?=\W)|$)");
    let pattern = Pattern::new_fancy("deny_list", regex_src, score).map_err(|_| RecognizerError::EmptyPatternList)?;
    PatternRecognizer::new(entity_type, vec![pattern])
}
