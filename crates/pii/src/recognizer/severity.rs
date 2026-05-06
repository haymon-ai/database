//! Severity tier attached to every recognizer.
//!
//! Total order is `Low < Medium < High < Critical` (declaration order via
//! derived [`Ord`]). Used as a floor in [`crate::analyzer::Analyzer`] builder
//! filters.

/// Closed set of severity tiers.
///
/// `#[non_exhaustive]` so additional tiers can be inserted without a major
/// version bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum Severity {
    /// Background risk; redact only when an explicit policy demands it.
    Low,
    /// Default tier; redact under broad category selections.
    Medium,
    /// Sensitive personal / financial / government identifier.
    High,
    /// Highest-risk data such as full card numbers, SSNs, secrets.
    Critical,
}

impl Severity {
    /// All variants in declaration order (low → critical).
    pub const ALL: &'static [Severity] = &[Severity::Low, Severity::Medium, Severity::High, Severity::Critical];
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_order_is_declaration_order() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn all_has_four_variants() {
        assert_eq!(Severity::ALL.len(), 4);
    }
}
