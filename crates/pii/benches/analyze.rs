//! Analyzer benches: recognizer match throughput, per-category cost, build cost.

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use dbmcp_pii::{AnalyzeOptions, Analyzer, Category};

mod common;

use common::{SIZES, mixed_payload};

fn bench_all_recognizers(c: &mut Criterion) {
    let analyzer = Analyzer::with_defaults();
    let opts = AnalyzeOptions::default();

    let mut group = c.benchmark_group("analyze/all_recognizers");
    for &size in SIZES {
        let payload = mixed_payload(size);
        group.throughput(Throughput::Bytes(payload.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &payload, |b, text| {
            b.iter(|| analyzer.analyze(black_box(text), black_box(&opts)));
        });
    }
    group.finish();
}

fn bench_by_category(c: &mut Criterion) {
    let opts = AnalyzeOptions::default();
    let payload = mixed_payload(64 * 1024);

    let mut group = c.benchmark_group("analyze/by_category");
    group.throughput(Throughput::Bytes(payload.len() as u64));
    for &cat in Category::ALL {
        let analyzer = Analyzer::builder()
            .categories([cat])
            .build()
            .expect("category has at least one recognizer");
        group.bench_with_input(BenchmarkId::from_parameter(cat.as_kebab()), &payload, |b, text| {
            b.iter(|| analyzer.analyze(black_box(text), black_box(&opts)));
        });
    }
    group.finish();
}

fn bench_analyzer_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyze/build");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(2));

    group.bench_function("with_defaults", |b| {
        b.iter(Analyzer::with_defaults);
    });

    group.bench_function("builder_filtered_financial", |b| {
        b.iter(|| {
            Analyzer::builder()
                .categories([Category::Financial])
                .build()
                .expect("build")
        });
    });
    group.finish();
}

criterion_group!(benches, bench_all_recognizers, bench_by_category, bench_analyzer_build,);
criterion_main!(benches);
