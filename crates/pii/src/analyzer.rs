//! Analyzer engine: registry + entry point + analyze-time options.

use std::collections::HashSet;
use std::time::Duration;

use crate::overlap;
use crate::recognizer::{EntityType, Recognizer};
use crate::result::RecognizerResult;
use crate::score::{MIN_SCORE, Score};

/// Per-call overrides handed to [`Analyzer::analyze`].
#[derive(Debug, Clone)]
pub struct AnalyzeOptions {
    /// Restrict the engine to recognizers whose `supported_entities` intersect this set.
    pub entity_allow_list: Option<HashSet<EntityType>>,
    /// Drop results whose score is below this floor before overlap resolution.
    pub min_score: Score,
    /// Per-pattern execution-time budget; `Fancy` patterns that exceed it are skipped.
    pub pattern_timeout: Duration,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            entity_allow_list: None,
            min_score: MIN_SCORE,
            pattern_timeout: Duration::from_millis(10),
        }
    }
}

/// Registry of recognizers and the public entry point for PII analysis.
#[derive(Default)]
pub struct Analyzer {
    recognizers: Vec<Box<dyn Recognizer>>,
}

impl std::fmt::Debug for Analyzer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self.recognizers.iter().map(|r| r.name()).collect();
        f.debug_struct("Analyzer").field("recognizers", &names).finish()
    }
}

impl Analyzer {
    /// Build an analyzer with no recognizers; caller registers their own.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build an analyzer pre-loaded with the eight v1 default recognizers.
    #[cfg(feature = "builtin")]
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut a = Self::empty();
        for r in crate::recognizer::builtin::all() {
            a.recognizers.push(Box::new(r));
        }
        a
    }

    /// Register a recognizer at the end of the registry.
    pub fn register(&mut self, recognizer: Box<dyn Recognizer>) -> &mut Self {
        self.recognizers.push(recognizer);
        self
    }

    /// Analyze `text`, returning merged + overlap-resolved results.
    #[must_use]
    pub fn analyze(&self, text: &str, opts: &AnalyzeOptions) -> Vec<RecognizerResult> {
        let mut results: Vec<RecognizerResult> = Vec::new();
        for recognizer in &self.recognizers {
            if !recognizer_in_allow_list(recognizer.as_ref(), opts.entity_allow_list.as_ref()) {
                continue;
            }
            for r in recognizer.analyze(text, opts) {
                if r.score < opts.min_score {
                    continue;
                }
                results.push(r);
            }
        }
        overlap::resolve(results)
    }
}

fn recognizer_in_allow_list(recognizer: &dyn Recognizer, allow: Option<&HashSet<EntityType>>) -> bool {
    let Some(allow) = allow else {
        return true;
    };
    recognizer.supported_entities().iter().any(|e| allow.contains(e))
}
