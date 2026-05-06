//! Frozen-contract tests for the catalog-expansion builder API (US1 / specs/095).
//!
//! - `with_defaults_is_eight`: FR-105 / Q1 — `with_defaults()` stays at the original 8.
//! - `tag_table_is_frozen`: every recognizer carries `(category, severity)` from contracts/public-api.md.
//! - `override_semantics`: explicit categories / `min_severity` apply on top of the merged registry.

use dbmcp_pii::{Analyzer, Category, Severity};

const V1_NAMES: &[&str] = &[
    "EMAIL_ADDRESS",
    "CREDIT_CARD",
    "IBAN_CODE",
    "IP_ADDRESS",
    "URL",
    "PHONE_NUMBER",
    "CRYPTO",
    "US_SSN",
];

fn entity_names(a: &Analyzer) -> Vec<String> {
    a.recognizers()
        .flat_map(|r| r.supported_entities().iter().map(|e| e.as_str().to_string()))
        .collect()
}

#[test]
fn with_defaults_is_eight() {
    let a = Analyzer::with_defaults();
    let got = entity_names(&a);
    let want: Vec<String> = V1_NAMES.iter().map(|s| (*s).to_string()).collect();
    assert_eq!(
        got, want,
        "with_defaults() must stay at the original 8 v1 recognizers (FR-105 / Q1)"
    );
}

#[test]
fn tag_table_is_frozen() {
    // Build via the builder so we exercise the merged registry; with `allow_empty_categories`
    // we tolerate categories without recognizers in this MVP slice (e.g. DigitalIdentity).
    let analyzer = Analyzer::builder()
        .categories(Category::ALL.iter().copied())
        .allow_empty_categories(true)
        .build()
        .expect("build");

    let mut tags: Vec<(String, Category, Severity)> = analyzer
        .recognizers()
        .flat_map(|r| {
            r.supported_entities()
                .iter()
                .map(|e| (e.as_str().to_string(), r.category(), r.severity()))
                .collect::<Vec<_>>()
        })
        .collect();
    tags.sort_by(|a, b| a.0.cmp(&b.0));

    // Frozen 8-row tag table for the v1 recognizers.
    let expected = vec![
        ("CREDIT_CARD".to_string(), Category::Financial, Severity::Critical),
        ("CRYPTO".to_string(), Category::Crypto, Severity::High),
        ("EMAIL_ADDRESS".to_string(), Category::Personal, Severity::High),
        ("IBAN_CODE".to_string(), Category::Financial, Severity::High),
        ("IP_ADDRESS".to_string(), Category::Network, Severity::Medium),
        ("PHONE_NUMBER".to_string(), Category::Contact, Severity::Medium),
        ("URL".to_string(), Category::Network, Severity::Low),
        ("US_SSN".to_string(), Category::Government, Severity::Critical),
    ];

    assert_eq!(tags, expected, "tag table drifted from contracts/public-api.md");
}

#[test]
fn override_semantics_neither_set_equals_with_defaults() {
    let a = Analyzer::builder().build().expect("build");
    assert_eq!(entity_names(&a), entity_names(&Analyzer::with_defaults()));
}

#[test]
fn categories_filter_registry() {
    // categories=[Network] with floor=Low keeps URL/IP_ADDRESS/MAC_ADDRESS,
    // drops Financial recognizers like CREDIT_CARD / IBAN_CODE.
    let a = Analyzer::builder()
        .categories([Category::Network])
        .min_severity(Severity::Low)
        .build()
        .expect("build");
    let names = entity_names(&a);
    assert!(
        names.contains(&"URL".to_string()),
        "URL should be present when categories=[Network]"
    );
    assert!(
        names.contains(&"IP_ADDRESS".to_string()),
        "IP_ADDRESS should be present"
    );
    assert!(
        !names.contains(&"CREDIT_CARD".to_string()),
        "Financial recognizers must drop when categories=[Network]"
    );
    assert!(
        !names.contains(&"IBAN_CODE".to_string()),
        "Financial recognizers must drop when categories=[Network]"
    );
}

#[test]
fn min_severity_filters_low_tier() {
    // floor=High drops URL (Low), IP_ADDRESS (Medium), PHONE_NUMBER (Medium).
    // EMAIL_ADDRESS (High) stays. allow_empty_categories(true) so Contact
    // (only PHONE_NUMBER@Medium and EMAIL_ADDRESS@High) doesn't error when
    // requested.
    let a = Analyzer::builder()
        .categories([
            Category::Personal,
            Category::Contact,
            Category::Government,
            Category::Financial,
            Category::Network,
            Category::DigitalIdentity,
        ])
        .min_severity(Severity::High)
        .allow_empty_categories(true)
        .build()
        .expect("build");
    let names = entity_names(&a);
    assert!(names.contains(&"EMAIL_ADDRESS".to_string()));
    assert!(
        !names.contains(&"URL".to_string()),
        "URL severity Low must drop when floor=High"
    );
    assert!(
        !names.contains(&"IP_ADDRESS".to_string()),
        "IP_ADDRESS severity Medium must drop when floor=High"
    );
    assert!(
        !names.contains(&"PHONE_NUMBER".to_string()),
        "PHONE_NUMBER severity Medium must drop when floor=High"
    );
}

#[test]
fn empty_category_errors_without_opt_out() {
    // Network category contains only Low/Medium recognizers (URL Low, IP_ADDRESS Medium,
    // MAC_ADDRESS Low). A Critical severity floor empties it entirely.
    let err = Analyzer::builder()
        .categories([Category::Network])
        .min_severity(Severity::Critical)
        .build()
        .unwrap_err();
    let dbmcp_pii::AnalyzerBuildError::EmptyCategory(cat) = err;
    assert_eq!(cat, Category::Network);
}

#[test]
fn empty_category_allowed_when_opt_in() {
    let a = Analyzer::builder()
        .categories([Category::Network])
        .min_severity(Severity::Critical)
        .allow_empty_categories(true)
        .build()
        .expect("build");
    assert!(entity_names(&a).is_empty());
}
