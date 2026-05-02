//! CT-009 / SC-005: a `Fancy` pattern that exceeds the per-pattern budget is
//! dropped and other recognizers continue to surface results.

use std::time::Duration;

use dbmcp_pii::{AnalyzeOptions, Analyzer, EntityType, Pattern, PatternRecognizer, Score, entity};

#[test]
fn fancy_pattern_dropped_on_timeout_other_recognizers_unaffected() {
    let mut analyzer = Analyzer::with_defaults();

    // Register a custom recognizer with a fancy pattern that the timeout will reliably trip.
    let bad = Pattern::new_fancy("evil", r"(?<![\w:])\d+", Score::new(0.7).unwrap()).unwrap();
    let custom = PatternRecognizer::new(EntityType::new("EVIL"), vec![bad]).unwrap();
    analyzer.register(Box::new(custom));

    let opts = AnalyzeOptions {
        pattern_timeout: Duration::from_micros(1),
        ..AnalyzeOptions::default()
    };

    let text = "ping me at jane.doe@example.com and 123";
    let results = analyzer.analyze(text, &opts);

    // The evil pattern's results are dropped (timeout), but email regex recognizer (regex-kind)
    // ignores the timeout entirely and still surfaces.
    assert!(
        results.iter().any(|r| r.entity_type == entity::EMAIL_ADDRESS),
        "email should still be detected: {results:?}"
    );
    assert!(
        results.iter().all(|r| r.entity_type != EntityType::new("EVIL")),
        "fancy timeout should drop EVIL: {results:?}"
    );
}
