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
use erltf::borrowed::BorrowedTerm;
use erltf::types::{Atom, BigInt, ExternalPid, ExternalPort, ExternalReference};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;

#[test]
fn test_borrowed_owned_equivalence_basic_types() {
    let owned_int = OwnedTerm::integer(42);
    let borrowed_int = BorrowedTerm::Integer(42);

    let owned_float = OwnedTerm::float(3.14);
    let borrowed_float = BorrowedTerm::Float(3.14);

    assert_eq!(
        owned_int.cmp(&owned_float),
        borrowed_int.cmp(&borrowed_float)
    );
}

#[test]
fn test_borrowed_owned_equivalence_strings_vs_binaries() {
    let owned_str = OwnedTerm::string("hello");
    let owned_bin = OwnedTerm::binary(b"hello".to_vec());

    let borrowed_str = BorrowedTerm::String(Cow::Borrowed("hello"));
    let borrowed_bin = BorrowedTerm::Binary(Cow::Borrowed(b"hello"));

    assert_eq!(owned_str.cmp(&owned_bin), borrowed_str.cmp(&borrowed_bin));
}

#[test]
fn test_borrowed_owned_equivalence_type_ordering() {
    let owned_terms = vec![
        OwnedTerm::binary(vec![1]),
        OwnedTerm::integer(5),
        OwnedTerm::atom("test"),
        OwnedTerm::float(2.5),
        OwnedTerm::tuple(vec![]),
        OwnedTerm::list(vec![OwnedTerm::integer(1)]),
    ];

    let borrowed_terms = vec![
        BorrowedTerm::Binary(Cow::Borrowed(&[1])),
        BorrowedTerm::Integer(5),
        BorrowedTerm::Atom(Cow::Borrowed("test")),
        BorrowedTerm::Float(2.5),
        BorrowedTerm::Tuple(vec![]),
        BorrowedTerm::List(vec![BorrowedTerm::Integer(1)]),
    ];

    for i in 0..owned_terms.len() {
        for j in 0..owned_terms.len() {
            assert_eq!(
                owned_terms[i].cmp(&owned_terms[j]),
                borrowed_terms[i].cmp(&borrowed_terms[j]),
                "Mismatch at indices {} vs {}",
                i,
                j
            );
        }
    }
}

#[test]
fn test_nan_ordering() {
    let nan1 = BorrowedTerm::Float(f64::NAN);
    let nan2 = BorrowedTerm::Float(f64::NAN);
    let regular = BorrowedTerm::Float(1.0);

    assert_eq!(nan1.cmp(&nan2), Ordering::Equal);
    assert_eq!(nan1.cmp(&regular), Ordering::Greater);
    assert_eq!(regular.cmp(&nan1), Ordering::Less);
}

#[test]
fn test_bigint_ordering_same_sign() {
    let small = BorrowedTerm::BigInt(BigInt::new(false, vec![1, 0]));
    let large = BorrowedTerm::BigInt(BigInt::new(false, vec![1, 0, 0]));

    assert_eq!(small.cmp(&large), Ordering::Less);
}

#[test]
fn test_bigint_ordering_different_signs() {
    let positive = BorrowedTerm::BigInt(BigInt::new(false, vec![1]));
    let negative = BorrowedTerm::BigInt(BigInt::new(true, vec![1]));

    assert_eq!(positive.cmp(&negative), Ordering::Greater);
    assert_eq!(negative.cmp(&positive), Ordering::Less);
}

#[test]
fn test_bigint_vs_integer() {
    let small_int = BorrowedTerm::Integer(100);
    let large_bigint = BorrowedTerm::BigInt(BigInt::new(false, vec![0, 0, 0, 0, 0, 0, 0, 1]));

    assert_eq!(small_int.cmp(&large_bigint), Ordering::Less);
}

#[test]
fn test_integer_min_edge_case() {
    let min_int = BorrowedTerm::Integer(i64::MIN);
    let negative_bigint = BorrowedTerm::BigInt(BigInt::new(
        true,
        vec![255, 255, 255, 255, 255, 255, 255, 127, 1],
    ));

    assert_eq!(min_int.cmp(&negative_bigint), Ordering::Greater);
}

#[test]
fn test_improper_list_ordering() {
    let list1 = BorrowedTerm::ImproperList {
        elements: vec![BorrowedTerm::Integer(1)],
        tail: Box::new(BorrowedTerm::Integer(2)),
    };

    let list2 = BorrowedTerm::ImproperList {
        elements: vec![BorrowedTerm::Integer(1)],
        tail: Box::new(BorrowedTerm::Integer(3)),
    };

    assert_eq!(list1.cmp(&list2), Ordering::Less);
}

#[test]
fn test_empty_list_vs_nil() {
    let empty_list = BorrowedTerm::List(vec![]);
    let nil = BorrowedTerm::Nil;

    assert_eq!(empty_list.cmp(&nil), Ordering::Equal);
    assert_eq!(nil.cmp(&empty_list), Ordering::Equal);
}

#[test]
fn test_non_empty_list_vs_nil() {
    let list = BorrowedTerm::List(vec![BorrowedTerm::Integer(1)]);
    let nil = BorrowedTerm::Nil;

    assert_eq!(list.cmp(&nil), Ordering::Greater);
    assert_eq!(nil.cmp(&list), Ordering::Less);
}

#[test]
fn test_map_ordering_borrowed() {
    let mut map1 = BTreeMap::new();
    map1.insert(
        BorrowedTerm::Atom(Cow::Borrowed("a")),
        BorrowedTerm::Integer(1),
    );

    let mut map2 = BTreeMap::new();
    map2.insert(
        BorrowedTerm::Atom(Cow::Borrowed("b")),
        BorrowedTerm::Integer(1),
    );

    let term1 = BorrowedTerm::Map(map1);
    let term2 = BorrowedTerm::Map(map2);

    assert_eq!(term1.cmp(&term2), Ordering::Less);
}

#[test]
fn test_reference_ordering() {
    let ref1 = BorrowedTerm::Reference(ExternalReference {
        node: Atom::new("node1"),
        creation: 1,
        ids: vec![1, 2, 3],
    });

    let ref2 = BorrowedTerm::Reference(ExternalReference {
        node: Atom::new("node1"),
        creation: 1,
        ids: vec![1, 2, 4],
    });

    assert_eq!(ref1.cmp(&ref2), Ordering::Less);
}

#[test]
fn test_pid_ordering() {
    let pid1 = BorrowedTerm::Pid(ExternalPid {
        node: Atom::new("node1"),
        id: 1,
        serial: 0,
        creation: 1,
    });

    let pid2 = BorrowedTerm::Pid(ExternalPid {
        node: Atom::new("node1"),
        id: 2,
        serial: 0,
        creation: 1,
    });

    assert_eq!(pid1.cmp(&pid2), Ordering::Less);
}

#[test]
fn test_port_ordering() {
    let port1 = BorrowedTerm::Port(ExternalPort {
        node: Atom::new("node1"),
        id: 1,
        creation: 1,
    });

    let port2 = BorrowedTerm::Port(ExternalPort {
        node: Atom::new("node1"),
        id: 2,
        creation: 1,
    });

    assert_eq!(port1.cmp(&port2), Ordering::Less);
}

#[test]
fn test_bit_binary_ordering() {
    let bb1 = BorrowedTerm::BitBinary {
        bytes: Cow::Borrowed(&[1, 2, 3]),
        bits: 5,
    };

    let bb2 = BorrowedTerm::BitBinary {
        bytes: Cow::Borrowed(&[1, 2, 3]),
        bits: 6,
    };

    assert_eq!(bb1.cmp(&bb2), Ordering::Less);
}

#[test]
fn test_tuple_ordering_by_length_then_content() {
    let t1 = BorrowedTerm::Tuple(vec![BorrowedTerm::Integer(1)]);
    let t2 = BorrowedTerm::Tuple(vec![BorrowedTerm::Integer(1), BorrowedTerm::Integer(2)]);
    let t3 = BorrowedTerm::Tuple(vec![BorrowedTerm::Integer(2)]);

    assert_eq!(t1.cmp(&t2), Ordering::Less);
    assert_eq!(t1.cmp(&t3), Ordering::Less);
}

#[test]
fn test_bigint_larger_than_8_bytes_vs_integer() {
    let large_bigint = BorrowedTerm::BigInt(BigInt::new(false, vec![0, 0, 0, 0, 0, 0, 0, 0, 1]));
    let max_int = BorrowedTerm::Integer(i64::MAX);

    assert_eq!(large_bigint.cmp(&max_int), Ordering::Greater);
}

#[test]
fn test_negative_bigint_larger_than_8_bytes_vs_integer() {
    let large_negative_bigint =
        BorrowedTerm::BigInt(BigInt::new(true, vec![0, 0, 0, 0, 0, 0, 0, 0, 1]));
    let min_int = BorrowedTerm::Integer(i64::MIN);

    assert_eq!(large_negative_bigint.cmp(&min_int), Ordering::Less);
}
