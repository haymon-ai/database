//! Anonymizer engine: collapse overlaps, rewrite right-to-left, emit audit trail.

use std::borrow::Cow;
use std::collections::HashMap;

use crate::operator::Operator;
use crate::overlap;
use crate::recognizer::EntityType;
use crate::result::{OperatorResult, RecognizerResult};

/// Per-entity-type operator map handed to [`Anonymizer::anonymize`].
#[derive(Debug, Clone)]
pub struct OperatorConfig {
    /// Explicit overrides, looked up by entity type.
    pub per_entity: HashMap<EntityType, Operator>,
    /// Fallback used when an entity type has no override (FR-017).
    pub default: Operator,
}

impl Default for OperatorConfig {
    fn default() -> Self {
        Self {
            per_entity: HashMap::new(),
            default: Operator::Replace {
                new_value: Cow::Borrowed("<REDACTED>"),
            },
        }
    }
}

/// Output of [`Anonymizer::anonymize`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnonymizedText {
    /// Rewritten text.
    pub text: String,
    /// Operator audit trail in original-position order.
    pub operations: Vec<OperatorResult>,
}

/// Engine that applies an [`OperatorConfig`] to a text + analyzer results.
#[derive(Debug, Default)]
pub struct Anonymizer;

impl Anonymizer {
    /// Construct a new anonymizer engine; stateless across calls.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Apply per-entity operators; return rewritten text plus audit trail.
    #[must_use]
    pub fn anonymize(&self, text: &str, results: Vec<RecognizerResult>, config: &OperatorConfig) -> AnonymizedText {
        // 1. Collapse overlaps (FR-016 reuses analyzer's resolution rules).
        let mut surviving = overlap::resolve(results);
        if surviving.is_empty() {
            return AnonymizedText {
                text: text.to_owned(),
                operations: Vec::new(),
            };
        }

        // 2. Sort by start ascending so we walk in reading order.
        surviving.sort_by_key(|r| r.start);

        // 3. Right-to-left rewrite + per-splice delta updates.
        let mut new_text = text.to_owned();
        let mut emissions: Vec<OperatorResult> = Vec::with_capacity(surviving.len());

        for result in surviving.iter().rev() {
            let RecognizerResult {
                ref entity_type,
                start,
                end,
                ..
            } = *result;
            if !new_text.is_char_boundary(start) || !new_text.is_char_boundary(end) {
                continue;
            }
            let candidate = &text[start..end];
            let operator = config
                .per_entity
                .get(entity_type)
                .cloned()
                .unwrap_or_else(|| Operator::default_for(entity_type));
            let new_value = operator.apply(candidate);
            let new_len = new_value.len();
            new_text.replace_range(start..end, &new_value);

            let new_start = start;
            let new_end = new_start + new_len;
            let original_len = end - start;
            let growth = new_len.saturating_sub(original_len);
            let shrink = original_len.saturating_sub(new_len);
            for prior in &mut emissions {
                prior.new_start = prior.new_start.saturating_add(growth).saturating_sub(shrink);
                prior.new_end = prior.new_end.saturating_add(growth).saturating_sub(shrink);
            }

            emissions.push(OperatorResult {
                entity_type: entity_type.clone(),
                operator: operator.kind(),
                original_start: start,
                original_end: end,
                new_start,
                new_end,
            });
        }

        emissions.reverse(); // back to original-position order

        AnonymizedText {
            text: new_text,
            operations: emissions,
        }
    }
}
