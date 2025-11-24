// Copyright (C) 2025-2026 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::Throughput;
use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use erltf::OwnedTerm;
use erltf::decode;
use erltf::encode;
use erltf::term_tuple;
use std::collections::BTreeMap;
use std::time::Duration;

fn create_large_nested_structure() -> OwnedTerm {
    let mut outer_map = BTreeMap::new();
    for i in 0..100 {
        let mut inner_map = BTreeMap::new();
        for j in 0..50 {
            inner_map.insert(
                OwnedTerm::atom(format!("key_{}", j)),
                term_tuple![
                    OwnedTerm::integer(i * 1000 + j),
                    OwnedTerm::float((i as f64) * 1.5 + (j as f64)),
                    OwnedTerm::binary(vec![i as u8; 64]),
                    OwnedTerm::string(format!("value_{}_{}", i, j)),
                ],
            );
        }
        outer_map.insert(OwnedTerm::integer(i), OwnedTerm::Map(inner_map));
    }
    OwnedTerm::Map(outer_map)
}

fn create_large_list() -> OwnedTerm {
    let elements: Vec<OwnedTerm> = (0..10000)
        .map(|i| {
            term_tuple![
                OwnedTerm::atom(format!("item_{}", i)),
                OwnedTerm::integer(i),
                OwnedTerm::float(i as f64 * 2.5),
            ]
        })
        .collect();
    OwnedTerm::List(elements)
}

fn create_large_binary() -> OwnedTerm {
    OwnedTerm::binary(vec![42u8; 1024 * 1024])
}

fn create_deep_nested_structure() -> OwnedTerm {
    let mut term = OwnedTerm::integer(0);
    for i in 0..100 {
        term = term_tuple![
            OwnedTerm::atom(format!("level_{}", i)),
            term,
            OwnedTerm::float(i as f64),
        ];
    }
    term
}

fn decode_large_nested_structure(c: &mut Criterion) {
    let term = create_large_nested_structure();
    let encoded = encode(&term).unwrap();

    let mut group = c.benchmark_group("decode_large_nested_structure");
    group.throughput(Throughput::Bytes(encoded.len() as u64));
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("nested_map_100x50", |b| {
        b.iter(|| decode(black_box(&encoded)).unwrap())
    });
    group.finish();
}

fn decode_large_list(c: &mut Criterion) {
    let term = create_large_list();
    let encoded = encode(&term).unwrap();

    let mut group = c.benchmark_group("decode_large_list");
    group.throughput(Throughput::Bytes(encoded.len() as u64));
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("list_10000_tuples", |b| {
        b.iter(|| decode(black_box(&encoded)).unwrap())
    });
    group.finish();
}

fn decode_large_binary(c: &mut Criterion) {
    let term = create_large_binary();
    let encoded = encode(&term).unwrap();

    let mut group = c.benchmark_group("decode_large_binary");
    group.throughput(Throughput::Bytes(encoded.len() as u64));
    group.bench_function("binary_1mb", |b| {
        b.iter(|| decode(black_box(&encoded)).unwrap())
    });
    group.finish();
}

fn decode_deep_nested_structure(c: &mut Criterion) {
    let term = create_deep_nested_structure();
    let encoded = encode(&term).unwrap();

    let mut group = c.benchmark_group("decode_deep_nested_structure");
    group.throughput(Throughput::Bytes(encoded.len() as u64));
    group.bench_function("depth_100", |b| {
        b.iter(|| decode(black_box(&encoded)).unwrap())
    });
    group.finish();
}

fn decode_small_structures(c: &mut Criterion) {
    let sizes = [10, 100, 1000, 100_000, 1_000_000];
    let mut group = c.benchmark_group("decode_small_structures");
    group.measurement_time(Duration::from_secs(15));

    for size in sizes.iter() {
        let terms: Vec<OwnedTerm> = (0..*size)
            .map(|i| {
                term_tuple![
                    OwnedTerm::atom("item"),
                    OwnedTerm::integer(i),
                    OwnedTerm::float(i as f64),
                ]
            })
            .collect();

        let encoded_terms: Vec<Vec<u8>> = terms.iter().map(|t| encode(t).unwrap()).collect();

        let total_size: usize = encoded_terms.iter().map(|e| e.len()).sum();

        group.throughput(Throughput::Bytes(total_size as u64));
        group.bench_with_input(
            BenchmarkId::new("tuples", size),
            &encoded_terms,
            |b, encoded_terms| {
                b.iter(|| {
                    for encoded in encoded_terms {
                        black_box(decode(black_box(encoded)).unwrap());
                    }
                })
            },
        );
    }

    group.finish();
}

fn decode_atom_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_atoms");

    let short_atom = encode(&OwnedTerm::atom("ok")).unwrap();
    let medium_atom = encode(&OwnedTerm::atom("medium_length_atom_name")).unwrap();
    let long_atom = encode(&OwnedTerm::atom(
        "this_is_a_very_long_atom_name_that_tests_performance_with_longer_strings",
    ))
    .unwrap();

    group.bench_function("short_atom", |b| {
        b.iter(|| decode(black_box(&short_atom)).unwrap())
    });

    group.bench_function("medium_atom", |b| {
        b.iter(|| decode(black_box(&medium_atom)).unwrap())
    });

    group.bench_function("long_atom", |b| {
        b.iter(|| decode(black_box(&long_atom)).unwrap())
    });

    group.finish();
}

fn decode_integer_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_integers");

    let small_int = encode(&OwnedTerm::integer(42)).unwrap();
    let medium_int = encode(&OwnedTerm::integer(1_000_000)).unwrap();
    let large_int = encode(&OwnedTerm::integer(i64::MAX)).unwrap();

    group.bench_function("small_int", |b| {
        b.iter(|| decode(black_box(&small_int)).unwrap())
    });

    group.bench_function("medium_int", |b| {
        b.iter(|| decode(black_box(&medium_int)).unwrap())
    });

    group.bench_function("large_int", |b| {
        b.iter(|| decode(black_box(&large_int)).unwrap())
    });

    group.finish();
}

fn decode_map_sizes(c: &mut Criterion) {
    let sizes = [10, 100, 1000, 100_000];
    let mut group = c.benchmark_group("decode_maps");
    group.measurement_time(Duration::from_secs(10));

    for size in sizes.iter() {
        let mut map = BTreeMap::new();
        for i in 0..*size {
            map.insert(OwnedTerm::atom(format!("key_{}", i)), OwnedTerm::integer(i));
        }
        let encoded = encode(&OwnedTerm::Map(map)).unwrap();

        group.throughput(Throughput::Bytes(encoded.len() as u64));
        group.bench_with_input(BenchmarkId::new("entries", size), &encoded, |b, encoded| {
            b.iter(|| decode(black_box(encoded)).unwrap())
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    decode_large_nested_structure,
    decode_large_list,
    decode_large_binary,
    decode_deep_nested_structure,
    decode_small_structures,
    decode_atom_variations,
    decode_integer_variations,
    decode_map_sizes
);
criterion_main!(benches);
