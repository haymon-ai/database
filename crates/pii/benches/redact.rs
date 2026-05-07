//! Redactor benches: end-to-end JSON walker on production-shaped payloads.

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use dbmcp_pii::Redactor;
use serde_json::{Map, Value, json};

mod common;

use common::corpus_positives;

fn pii_pool() -> Vec<String> {
    let mut pool = Vec::new();
    for name in ["email.txt", "ip.txt", "credit_card.txt", "iban.txt"] {
        pool.extend(corpus_positives(name));
    }
    pool
}

fn flat_rows() -> Vec<Value> {
    const ROWS: usize = 1000;
    const COLS: usize = 10;
    let pool = pii_pool();
    let mut out = Vec::with_capacity(ROWS);
    for r in 0..ROWS {
        let mut map = Map::new();
        for c in 0..COLS {
            let key = format!("col_{c}");
            let cell = if (r * COLS + c).is_multiple_of(3) {
                pool[(r + c) % pool.len()].clone()
            } else {
                format!("plain text cell {r}/{c} no pii here")
            };
            map.insert(key, Value::String(cell));
        }
        out.push(Value::Object(map));
    }
    out
}

fn nested_jsonb_rows() -> Vec<Value> {
    const ROWS: usize = 100;
    let pool = pii_pool();
    let mut out = Vec::with_capacity(ROWS);
    for r in 0..ROWS {
        let leaf_pii = pool[r % pool.len()].clone();
        let leaf_plain = format!("user_event_{r}");
        let payload = json!({
            "id": r,
            "ts": "2026-05-07T10:00:00Z",
            "data": {
                "user": {
                    "profile": {
                        "contact": leaf_pii,
                        "label": leaf_plain,
                    },
                    "tags": ["alpha", "beta", pool[(r + 1) % pool.len()].clone()],
                },
                "audit": [
                    {"event": "login", "src": pool[(r + 2) % pool.len()].clone()},
                    {"event": "view", "src": "192.0.2.0"},
                ],
            },
        });
        out.push(payload);
    }
    out
}

fn large_blob_rows() -> Vec<Value> {
    const ROWS: usize = 10;
    const TARGET: usize = 64 * 1024;
    let pool = pii_pool();
    let mut out = Vec::with_capacity(ROWS);
    for r in 0..ROWS {
        let mut blob = String::with_capacity(TARGET + 256);
        let mut i = 0usize;
        while blob.len() < TARGET {
            blob.push_str("lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor ");
            blob.push_str(&pool[(r + i) % pool.len()]);
            blob.push(' ');
            i += 1;
        }
        out.push(json!({"row": r, "blob": blob}));
    }
    out
}

fn bench_redact_shapes(c: &mut Criterion) {
    let redactor = Redactor::with_defaults();

    let shapes: [(&str, Vec<Value>); 3] = [
        ("flat_rows", flat_rows()),
        ("nested_jsonb", nested_jsonb_rows()),
        ("large_blob", large_blob_rows()),
    ];

    let mut group = c.benchmark_group("redact/shapes");
    for (label, rows) in &shapes {
        group.throughput(Throughput::Elements(rows.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(label), rows, |b, rs| {
            b.iter_batched(
                || rs.clone(),
                |mut r| {
                    redactor
                        .apply(black_box(&mut r))
                        .expect("redactor must not panic on bench input")
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_redact_shapes);
criterion_main!(benches);
