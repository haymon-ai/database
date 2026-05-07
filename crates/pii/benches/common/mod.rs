//! Shared helpers for the PII benches: payload synthesis on top of the
//! shared `test_support::Corpus` loader.

#![allow(dead_code)]

use dbmcp_pii::corpus::Corpus;
use dbmcp_pii::{AnalyzeOptions, Analyzer, RecognizerResult};

/// Input sizes (bytes) swept by the throughput benches.
pub const SIZES: &[usize] = &[1024, 8 * 1024, 64 * 1024, 512 * 1024];

const FILLER: &str = "the quick brown fox jumps over the lazy dog while logs ship and metrics tick along the wire ";

/// Build a deterministic payload of approximately `size_bytes` bytes by
/// interleaving filler prose with corpus positives modulo the input slice.
#[must_use]
pub fn synth_payload(size_bytes: usize, positives: &[String]) -> String {
    assert!(!positives.is_empty(), "positives must not be empty");
    let mut out = String::with_capacity(size_bytes + 256);
    let mut i = 0usize;
    while out.len() < size_bytes {
        out.push_str(FILLER);
        out.push_str(&positives[i % positives.len()]);
        out.push(' ');
        i += 1;
    }
    out
}

/// Build a mixed payload using positives from several corpora.
#[must_use]
pub fn mixed_payload(size_bytes: usize) -> String {
    let mut all: Vec<String> = Vec::new();
    for stem in ["email", "credit_card", "iban", "ip", "url"] {
        all.extend(Corpus::load(stem).positives);
    }
    synth_payload(size_bytes, &all)
}

/// Pre-compute analyzer results for `text` so anonymizer benches don't pay
/// recognition cost on the hot path.
#[must_use]
pub fn sample_results(analyzer: &Analyzer, text: &str) -> Vec<RecognizerResult> {
    analyzer.analyze(text, &AnalyzeOptions::default())
}
