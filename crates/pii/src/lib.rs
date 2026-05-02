//! PII analyzer and anonymizer for `dbmcp`.
//!
//! Library-only crate. Ports Presidio's language-agnostic recognizer and
//! anonymizer pipeline to Rust: regex/pattern recognition with optional
//! checksum validation, plus four built-in operators (`Replace`, `Mask`,
//! `Redact`, `Hash`). No NLP, no LLM, no network. Not wired into the MCP
//! server in this iteration.
//!
//! See `specs/082-pii-pattern-recognizers/` for the source spec.
//!
//! # Quickstart
//!
//! ```
//! use dbmcp_pii::{AnalyzeOptions, Analyzer, Anonymizer, OperatorConfig};
//!
//! let analyzer = Analyzer::with_defaults();
//! let anonymizer = Anonymizer::new();
//! let text = "ping me at jane.doe@example.com";
//! let results = analyzer.analyze(text, &AnalyzeOptions::default());
//! let out = anonymizer.anonymize(text, results, &OperatorConfig::default());
//! assert_eq!(out.text, "ping me at <EMAIL_ADDRESS>");
//! ```

#![deny(missing_docs)]

pub mod analyzer;
pub mod anonymizer;
pub mod error;
pub mod operator;
pub mod overlap;
pub mod pattern;
pub mod recognizer;
pub mod result;
pub mod score;
pub mod timeout;

pub use crate::analyzer::{AnalyzeOptions, Analyzer};
pub use crate::anonymizer::{AnonymizedText, Anonymizer, OperatorConfig};
pub use crate::error::{AnalyzerError, OperatorError, PatternError, RecognizerError};
pub use crate::operator::{ChunkCount, HashAlgorithm, Operator, OperatorKind};
pub use crate::pattern::{Pattern, PatternKind};
pub use crate::recognizer::{
    EntityType, PatternRecognizer, Recognizer, ValidationOutcome, Validator, deny_list_recognizer,
};
pub use crate::result::{AnalysisExplanation, OperatorResult, RecognizerResult};
pub use crate::score::{MAX_SCORE, MIN_SCORE, Score};

#[cfg(feature = "builtin")]
pub use crate::recognizer::builtin;
#[cfg(feature = "builtin")]
pub use crate::recognizer::entity;
