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

use erltf::OwnedTerm;
use erltf::types::BigInt;
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[test]
fn test_erlang_term_ordering_types() {
    let number = OwnedTerm::integer(1);
    let float = OwnedTerm::float(1.5);
    let atom = OwnedTerm::atom("test");
    let tuple = OwnedTerm::tuple(vec![]);
    let list = OwnedTerm::list(vec![]);
    let binary = OwnedTerm::binary(vec![1, 2, 3]);

    assert!(number < float);
    assert!(float < atom);
    assert!(atom < tuple);
    assert!(tuple < list);
    assert!(list < binary);
}

#[test]
fn test_number_float_value_comparison() {
    assert!(OwnedTerm::integer(1) < OwnedTerm::float(1.5));
    assert!(OwnedTerm::integer(10) > OwnedTerm::float(5.0));
    assert_eq!(
        OwnedTerm::integer(1).cmp(&OwnedTerm::float(1.0)),
        Ordering::Equal
    );
    assert!(OwnedTerm::float(0.5) < OwnedTerm::integer(1));
    assert!(OwnedTerm::float(100.5) > OwnedTerm::integer(100));
}

#[test]
fn test_integer_ordering() {
    assert!(OwnedTerm::integer(-100) < OwnedTerm::integer(0));
    assert!(OwnedTerm::integer(0) < OwnedTerm::integer(100));
    assert_eq!(
        OwnedTerm::integer(42).cmp(&OwnedTerm::integer(42)),
        Ordering::Equal
    );
}

#[test]
fn test_float_ordering() {
    assert!(OwnedTerm::float(-1.5) < OwnedTerm::float(0.0));
    assert!(OwnedTerm::float(0.0) < OwnedTerm::float(1.5));
    assert!(OwnedTerm::float(1.0) < OwnedTerm::float(1.1));
}

#[test]
fn test_atom_ordering() {
    assert!(OwnedTerm::atom("abc") < OwnedTerm::atom("def"));
    assert!(OwnedTerm::atom("test") < OwnedTerm::atom("testing"));
    assert_eq!(
        OwnedTerm::atom("same").cmp(&OwnedTerm::atom("same")),
        Ordering::Equal
    );
}

#[test]
fn test_tuple_ordering() {
    let t1 = OwnedTerm::tuple(vec![OwnedTerm::integer(1)]);
    let t2 = OwnedTerm::tuple(vec![OwnedTerm::integer(1), OwnedTerm::integer(2)]);
    let t3 = OwnedTerm::tuple(vec![OwnedTerm::integer(2)]);

    assert!(t1 < t2);
    assert!(t1 < t3);
}

#[test]
fn test_tuple_ordering_by_elements() {
    let t1 = OwnedTerm::tuple(vec![OwnedTerm::integer(1), OwnedTerm::atom("a")]);
    let t2 = OwnedTerm::tuple(vec![OwnedTerm::integer(1), OwnedTerm::atom("b")]);
    let t3 = OwnedTerm::tuple(vec![OwnedTerm::integer(2), OwnedTerm::atom("a")]);

    assert!(t1 < t2);
    assert!(t1 < t3);
}

#[test]
fn test_list_ordering() {
    let l1 = OwnedTerm::list(vec![OwnedTerm::integer(1)]);
    let l2 = OwnedTerm::list(vec![OwnedTerm::integer(1), OwnedTerm::integer(2)]);
    let l3 = OwnedTerm::list(vec![OwnedTerm::integer(2)]);

    assert!(l1 < l2);
    assert!(l1 < l3);
}

#[test]
fn test_list_vs_nil() {
    let empty_list = OwnedTerm::list(vec![]);
    let nil = OwnedTerm::nil();

    assert_eq!(empty_list.cmp(&nil), Ordering::Equal);
}

#[test]
fn test_binary_ordering() {
    let b1 = OwnedTerm::binary(vec![1, 2, 3]);
    let b2 = OwnedTerm::binary(vec![1, 2, 3, 4]);
    let b3 = OwnedTerm::binary(vec![1, 2, 4]);

    assert!(b1 < b2);
    assert!(b1 < b3);
}

#[test]
fn test_map_ordering_deterministic() {
    let mut map1 = BTreeMap::new();
    map1.insert(OwnedTerm::atom("z"), OwnedTerm::integer(1));
    map1.insert(OwnedTerm::atom("a"), OwnedTerm::integer(2));
    map1.insert(OwnedTerm::atom("m"), OwnedTerm::integer(3));

    let mut map2 = BTreeMap::new();
    map2.insert(OwnedTerm::atom("a"), OwnedTerm::integer(2));
    map2.insert(OwnedTerm::atom("m"), OwnedTerm::integer(3));
    map2.insert(OwnedTerm::atom("z"), OwnedTerm::integer(1));

    assert_eq!(
        OwnedTerm::Map(map1).cmp(&OwnedTerm::Map(map2)),
        Ordering::Equal
    );
}

#[test]
fn test_map_ordering_by_size() {
    let mut small_map = BTreeMap::new();
    small_map.insert(OwnedTerm::atom("a"), OwnedTerm::integer(1));

    let mut large_map = BTreeMap::new();
    large_map.insert(OwnedTerm::atom("a"), OwnedTerm::integer(1));
    large_map.insert(OwnedTerm::atom("b"), OwnedTerm::integer(2));

    assert!(OwnedTerm::Map(small_map) < OwnedTerm::Map(large_map));
}

#[test]
fn test_map_ordering_by_keys() {
    let mut map1 = BTreeMap::new();
    map1.insert(OwnedTerm::atom("a"), OwnedTerm::integer(1));

    let mut map2 = BTreeMap::new();
    map2.insert(OwnedTerm::atom("b"), OwnedTerm::integer(1));

    assert!(OwnedTerm::Map(map1) < OwnedTerm::Map(map2));
}

#[test]
fn test_map_ordering_by_values() {
    let mut map1 = BTreeMap::new();
    map1.insert(OwnedTerm::atom("key"), OwnedTerm::integer(1));

    let mut map2 = BTreeMap::new();
    map2.insert(OwnedTerm::atom("key"), OwnedTerm::integer(2));

    assert!(OwnedTerm::Map(map1) < OwnedTerm::Map(map2));
}

#[test]
fn test_complex_map_ordering() {
    let mut map1 = BTreeMap::new();
    map1.insert(OwnedTerm::integer(1), OwnedTerm::atom("one"));
    map1.insert(OwnedTerm::atom("two"), OwnedTerm::integer(2));
    map1.insert(
        OwnedTerm::tuple(vec![OwnedTerm::integer(3)]),
        OwnedTerm::list(vec![OwnedTerm::atom("apple")]),
    );

    let mut map2 = BTreeMap::new();
    map2.insert(OwnedTerm::integer(1), OwnedTerm::atom("one"));
    map2.insert(OwnedTerm::atom("two"), OwnedTerm::integer(2));
    map2.insert(
        OwnedTerm::tuple(vec![OwnedTerm::integer(3)]),
        OwnedTerm::list(vec![OwnedTerm::atom("banana")]),
    );

    assert!(OwnedTerm::Map(map1) < OwnedTerm::Map(map2));
}

#[test]
fn test_sorting_mixed_terms() {
    let mut terms = [
        OwnedTerm::binary(vec![1]),
        OwnedTerm::integer(5),
        OwnedTerm::atom("test"),
        OwnedTerm::float(2.5),
        OwnedTerm::tuple(vec![]),
        OwnedTerm::list(vec![OwnedTerm::integer(1)]),
        OwnedTerm::integer(-5),
        OwnedTerm::float(10.0),
    ];

    terms.sort();

    assert_eq!(terms[0], OwnedTerm::integer(-5));
    assert_eq!(terms[1], OwnedTerm::float(2.5));
    assert_eq!(terms[2], OwnedTerm::integer(5));
    assert_eq!(terms[3], OwnedTerm::float(10.0));
    assert_eq!(terms[4], OwnedTerm::atom("test"));
    assert_eq!(terms[5], OwnedTerm::tuple(vec![]));
    assert_eq!(terms[6], OwnedTerm::list(vec![OwnedTerm::integer(1)]));
    assert_eq!(terms[7], OwnedTerm::binary(vec![1]));
}

#[test]
fn test_map_with_sorted_keys() {
    let mut map = BTreeMap::new();
    map.insert(OwnedTerm::integer(3), OwnedTerm::atom("three"));
    map.insert(OwnedTerm::integer(1), OwnedTerm::atom("one"));
    map.insert(OwnedTerm::integer(2), OwnedTerm::atom("two"));

    let term = OwnedTerm::Map(map);
    let encoded = erltf::encode(&term).unwrap();
    let decoded = erltf::decode(&encoded).unwrap();

    assert_eq!(term, decoded);
}

#[test]
fn test_comparison_transitivity() {
    let a = OwnedTerm::integer(1);
    let b = OwnedTerm::float(2.0);
    let c = OwnedTerm::atom("test");

    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
}

#[test]
fn test_bigint_float_comparison() {
    let small_bigint = OwnedTerm::BigInt(BigInt::new(false, vec![0, 0, 0, 0, 0, 0, 0, 128]));
    let large_float = OwnedTerm::float(1e20);
    let small_float = OwnedTerm::float(1.5);

    assert!(small_float < small_bigint);
    assert!(small_bigint < large_float);
}

#[test]
fn test_nan_ordering_owned() {
    let nan1 = OwnedTerm::float(f64::NAN);
    let nan2 = OwnedTerm::float(f64::NAN);
    let regular = OwnedTerm::float(1.0);

    assert_eq!(nan1.cmp(&nan2), Ordering::Equal);
    assert_eq!(nan1.cmp(&regular), Ordering::Greater);
    assert_eq!(regular.cmp(&nan1), Ordering::Less);
}

#[test]
fn test_string_vs_binary_ordering() {
    let string = OwnedTerm::string("hello");
    let binary = OwnedTerm::binary(b"hello".to_vec());

    assert_eq!(string.cmp(&binary), Ordering::Equal);
}

#[test]
fn test_string_vs_binary_different_values() {
    let string = OwnedTerm::string("apple");
    let binary = OwnedTerm::binary(b"banana".to_vec());

    assert!(string < binary);
}

#[test]
fn test_improper_list_ordering() {
    let list1 = OwnedTerm::improper_list(vec![OwnedTerm::integer(1)], OwnedTerm::integer(2));
    let list2 = OwnedTerm::improper_list(vec![OwnedTerm::integer(1)], OwnedTerm::integer(3));

    assert!(list1 < list2);
}

#[test]
fn test_bigint_ordering_large_numbers() {
    let bigint1 = OwnedTerm::BigInt(BigInt::new(false, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
    let bigint2 = OwnedTerm::BigInt(BigInt::new(false, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 11]));

    assert!(bigint1 < bigint2);
}

#[test]
fn test_integer_min_max_edge_cases() {
    let min = OwnedTerm::integer(i64::MIN);
    let max = OwnedTerm::integer(i64::MAX);
    let zero = OwnedTerm::integer(0);

    assert!(min < zero);
    assert!(zero < max);
    assert!(min < max);
}

#[test]
fn test_bit_binary_ordering() {
    let bb1 = OwnedTerm::BitBinary {
        bytes: vec![1, 2, 3],
        bits: 5,
    };
    let bb2 = OwnedTerm::BitBinary {
        bytes: vec![1, 2, 3],
        bits: 6,
    };
    let bb3 = OwnedTerm::BitBinary {
        bytes: vec![1, 2, 4],
        bits: 5,
    };

    assert!(bb1 < bb2);
    assert!(bb1 < bb3);
}

#[test]
fn test_bigint_larger_than_8_bytes_vs_integer() {
    let large_bigint = OwnedTerm::BigInt(BigInt::new(false, vec![0, 0, 0, 0, 0, 0, 0, 0, 1]));
    let max_int = OwnedTerm::integer(i64::MAX);

    assert!(large_bigint > max_int);
}

#[test]
fn test_negative_bigint_larger_than_8_bytes_vs_integer() {
    let large_negative_bigint =
        OwnedTerm::BigInt(BigInt::new(true, vec![0, 0, 0, 0, 0, 0, 0, 0, 1]));
    let min_int = OwnedTerm::integer(i64::MIN);

    assert!(large_negative_bigint < min_int);
}
