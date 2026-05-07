//! Shared helpers for the PII benches: corpus loader + payload synthesis.

#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use dbmcp_pii::{AnalyzeOptions, Analyzer, RecognizerResult};

/// Input sizes (bytes) swept by the throughput benches.
pub const SIZES: &[usize] = &[1024, 8 * 1024, 64 * 1024, 512 * 1024];

const FILLER: &str = "the quick brown fox jumps over the lazy dog while logs ship and metrics tick along the wire ";

/// Read the `# positive` section of `tests/corpus/{name}` into a `Vec<String>`.
///
/// # Panics
///
/// Panics if the corpus file does not exist; benches are run from the crate
/// root and the fixture set is checked into the repo.
pub fn corpus_positives(name: &str) -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("corpus")
        .join(name);
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read corpus {}: {e}", path.display()));
    let mut out = Vec::new();
    let mut in_positives = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.eq_ignore_ascii_case("# positive") {
            in_positives = true;
            continue;
        }
        if trimmed.eq_ignore_ascii_case("# negative") {
            in_positives = false;
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        if in_positives {
            out.push(trimmed.to_owned());
        }
    }
    out
}

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
    for name in ["email.txt", "credit_card.txt", "iban.txt", "ip.txt", "url.txt"] {
        all.extend(corpus_positives(name));
    }
    synth_payload(size_bytes, &all)
}

/// Pre-compute analyzer results for `text` so anonymizer benches don't pay
/// recognition cost on the hot path.
#[must_use]
pub fn sample_results(analyzer: &Analyzer, text: &str) -> Vec<RecognizerResult> {
    analyzer.analyze(text, &AnalyzeOptions::default())
}
