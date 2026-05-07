//! Anonymizer benches: operator comparison + variant tuning.

use std::borrow::Cow;
use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use dbmcp_pii::{Analyzer, ChunkCount, HashAlgorithm, Operator, OperatorConfig, anonymize};

mod common;

use common::{mixed_payload, sample_results};

const PAYLOAD_BYTES: usize = 16 * 1024;

fn build_cfg(default: Operator) -> OperatorConfig {
    OperatorConfig {
        per_entity: std::collections::HashMap::new(),
        default: Some(default),
    }
}

fn bench_operators(c: &mut Criterion) {
    let analyzer = Analyzer::with_defaults();
    let payload = mixed_payload(PAYLOAD_BYTES);
    let results = sample_results(&analyzer, &payload);

    let cases: [(&str, Operator); 4] = [
        (
            "replace",
            Operator::Replace {
                new_value: Cow::Borrowed("<REDACTED>"),
            },
        ),
        ("mask", Operator::default_mask()),
        ("redact", Operator::Redact),
        ("hash_sha256", Operator::hash(HashAlgorithm::Sha256)),
    ];

    let mut group = c.benchmark_group("anonymize/operators");
    group.throughput(Throughput::Bytes(payload.len() as u64));
    for (label, op) in cases {
        let cfg = build_cfg(op);
        group.bench_with_input(BenchmarkId::new(label, payload.len()), &payload, |b, text| {
            b.iter_batched(
                || results.clone(),
                |r| anonymize(black_box(text), r, black_box(&cfg)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_hash_algorithms(c: &mut Criterion) {
    let analyzer = Analyzer::with_defaults();
    let payload = mixed_payload(PAYLOAD_BYTES);
    let results = sample_results(&analyzer, &payload);

    let mut group = c.benchmark_group("anonymize/hash_algorithms");
    group.throughput(Throughput::Bytes(payload.len() as u64));
    for (label, algo) in [("sha256", HashAlgorithm::Sha256), ("sha512", HashAlgorithm::Sha512)] {
        let cfg = build_cfg(Operator::hash(algo));
        group.bench_with_input(BenchmarkId::from_parameter(label), &payload, |b, text| {
            b.iter_batched(
                || results.clone(),
                |r| anonymize(black_box(text), r, black_box(&cfg)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_mask_chunk_count(c: &mut Criterion) {
    let analyzer = Analyzer::with_defaults();
    let payload = mixed_payload(PAYLOAD_BYTES);
    let results = sample_results(&analyzer, &payload);

    let cases: [(&str, Operator); 3] = [
        (
            "all_from_end",
            Operator::Mask {
                masking_char: '*',
                chars_to_mask: ChunkCount::All,
                from_end: true,
            },
        ),
        (
            "n4_from_end",
            Operator::Mask {
                masking_char: '*',
                chars_to_mask: ChunkCount::N(4),
                from_end: true,
            },
        ),
        (
            "n4_from_start",
            Operator::Mask {
                masking_char: '*',
                chars_to_mask: ChunkCount::N(4),
                from_end: false,
            },
        ),
    ];

    let mut group = c.benchmark_group("anonymize/mask_chunk_count");
    group.throughput(Throughput::Bytes(payload.len() as u64));
    for (label, op) in cases {
        let cfg = build_cfg(op);
        group.bench_with_input(BenchmarkId::from_parameter(label), &payload, |b, text| {
            b.iter_batched(
                || results.clone(),
                |r| anonymize(black_box(text), r, black_box(&cfg)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_operators, bench_hash_algorithms, bench_mask_chunk_count);
criterion_main!(benches);
