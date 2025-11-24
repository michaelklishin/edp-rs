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
use erltf::types::{Atom, BigInt, ExternalPid, ExternalPort, ExternalReference};
use erltf::{decode, encode};
use std::collections::BTreeMap;

#[test]
fn test_encode_decode_small_integer() {
    let term = OwnedTerm::Integer(42);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_negative_integer() {
    let term = OwnedTerm::Integer(-12345);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_atom() {
    let term = OwnedTerm::Atom(Atom::new("test"));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_atom_utf8() {
    let term = OwnedTerm::Atom(Atom::new("тест"));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_float() {
    let term = OwnedTerm::Float(2.5);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_binary() {
    let term = OwnedTerm::Binary(vec![1, 2, 3, 4, 5]);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_string() {
    let term = OwnedTerm::String("hello world".to_string());
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(OwnedTerm::Binary(b"hello world".to_vec()), decoded);
}

#[test]
fn test_encode_decode_empty_list() {
    let term = OwnedTerm::List(vec![]);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(OwnedTerm::Nil, decoded);
}

#[test]
fn test_encode_decode_list() {
    let term = OwnedTerm::List(vec![
        OwnedTerm::Integer(1),
        OwnedTerm::Integer(2),
        OwnedTerm::Integer(3),
    ]);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_tuple() {
    let term = OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("ok")),
        OwnedTerm::Integer(42),
    ]);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_map() {
    let mut map = BTreeMap::new();
    map.insert(OwnedTerm::Atom(Atom::new("key1")), OwnedTerm::Integer(100));
    map.insert(
        OwnedTerm::Atom(Atom::new("key2")),
        OwnedTerm::Binary(b"value".to_vec()),
    );
    let term = OwnedTerm::Map(map);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_nil() {
    let term = OwnedTerm::Nil;
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_large_tuple() {
    let elements: Vec<OwnedTerm> = (0..300).map(OwnedTerm::Integer).collect();
    let term = OwnedTerm::Tuple(elements);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_nested_structure() {
    let inner_map = {
        let mut m = BTreeMap::new();
        m.insert(OwnedTerm::Atom(Atom::new("inner")), OwnedTerm::Integer(1));
        m
    };

    let term = OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("complex")),
        OwnedTerm::List(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::Integer(2),
            OwnedTerm::Integer(3),
        ]),
        OwnedTerm::Map(inner_map),
    ]);

    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_i64_min() {
    let term = OwnedTerm::Integer(i64::MIN);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    match decoded {
        OwnedTerm::BigInt(_) => (),
        _ => panic!("Expected BigInt, got {:?}", decoded),
    }
}

#[test]
fn test_encode_decode_i64_max() {
    let term = OwnedTerm::Integer(i64::MAX);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    match decoded {
        OwnedTerm::BigInt(_) => (),
        _ => panic!("Expected BigInt, got {:?}", decoded),
    }
}

#[test]
fn test_encode_decode_empty_atom() {
    let term = OwnedTerm::Atom(Atom::new(""));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_empty_binary() {
    let term = OwnedTerm::Binary(vec![]);
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_improper_list() {
    let term = OwnedTerm::ImproperList {
        elements: vec![OwnedTerm::Integer(1), OwnedTerm::Integer(2)],
        tail: Box::new(OwnedTerm::Atom(Atom::new("tail"))),
    };
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_bit_binary() {
    let term = OwnedTerm::BitBinary {
        bytes: vec![0xFF, 0xAA],
        bits: 3,
    };
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_external_pid() {
    let term = OwnedTerm::Pid(ExternalPid::new(Atom::new("node@host"), 12345, 67890, 4));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_external_port() {
    let term = OwnedTerm::Port(ExternalPort::new(Atom::new("node@host"), 98765, 4));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_external_reference() {
    let term = OwnedTerm::Reference(ExternalReference::new(
        Atom::new("node@host"),
        4,
        vec![111, 222, 333],
    ));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_bigint_positive() {
    let term = OwnedTerm::BigInt(BigInt::new(false, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_bigint_negative() {
    let term = OwnedTerm::BigInt(BigInt::new(true, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF]));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_decode_long_atom() {
    let long_name = "a".repeat(300);
    let term = OwnedTerm::Atom(Atom::new(long_name.clone()));
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_encode_atom_too_large() {
    let too_long = "a".repeat(70000);
    let term = OwnedTerm::Atom(Atom::new(too_long));
    let result = encode(&term);
    assert!(result.is_err());
}

#[test]
fn test_invalid_version() {
    let data = vec![132, 106];
    assert!(decode(&data).is_err());
}

#[test]
fn test_unknown_tag() {
    let data = vec![131, 255];
    assert!(decode(&data).is_err());
}

#[test]
fn test_bit_binary_invalid_bits_zero() {
    let data = vec![131, 77, 0, 0, 0, 1, 0, 0xFF];
    assert!(decode(&data).is_err());
}

#[test]
fn test_bit_binary_invalid_bits_nine() {
    let data = vec![131, 77, 0, 0, 0, 1, 9, 0xFF];
    assert!(decode(&data).is_err());
}

#[test]
fn test_bit_binary_empty_with_bits() {
    let data = vec![131, 77, 0, 0, 0, 0, 3];
    assert!(decode(&data).is_err());
}

#[test]
fn test_float_positive_infinity() {
    let pos_inf = OwnedTerm::Float(f64::INFINITY);
    let encoded = encode(&pos_inf).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, pos_inf);
}

#[test]
fn test_float_negative_infinity() {
    let neg_inf = OwnedTerm::Float(f64::NEG_INFINITY);
    let encoded = encode(&neg_inf).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, neg_inf);
}

#[test]
fn test_float_nan() {
    let nan = OwnedTerm::Float(f64::NAN);
    let encoded = encode(&nan).unwrap();
    let decoded = decode(&encoded).unwrap();
    match decoded {
        OwnedTerm::Float(f) if f.is_nan() => (),
        _ => panic!("Expected NaN"),
    }
}

#[test]
fn test_unexpected_eof_in_binary() {
    let data = vec![131, 109, 0, 0, 0, 10, 1, 2, 3];
    assert!(decode(&data).is_err());
}

#[test]
fn test_unexpected_eof_in_list() {
    let data = vec![131, 108, 0, 0, 0, 2, 97, 1];
    assert!(decode(&data).is_err());
}

#[test]
fn test_trailing_bytes() {
    let mut data = vec![131, 97, 42];
    data.extend_from_slice(&[99, 100, 101]);
    assert!(decode(&data).is_err());
}

#[test]
fn test_deeply_nested_tuples() {
    let mut term = OwnedTerm::integer(0);
    for _ in 0..100 {
        term = OwnedTerm::Tuple(vec![term]);
    }
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}

#[test]
fn test_deeply_nested_lists() {
    let mut term = OwnedTerm::integer(0);
    for _ in 0..100 {
        term = OwnedTerm::List(vec![term]);
    }
    let encoded = encode(&term).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(term, decoded);
}
