//! Entity-type newtype tagging the kind of PII a recognizer emits.

use std::borrow::Cow;

/// Tag identifying the kind of PII a recognizer emits.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EntityType(pub(crate) Cow<'static, str>);

impl EntityType {
    /// Build an entity type from any string-like source.
    #[must_use]
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }

    /// Return the entity-type name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
