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
use erltf_serde::{from_term, to_term};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Struct with OwnedTerm field

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Payload {
    op: u16,
    d: OwnedTerm,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Wrapper {
    data: OwnedTerm,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MixedResult {
    items: Vec<OwnedTerm>,
}

#[test]
fn test_struct_with_owned_term_field_integer_payload() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Integer(42),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.op, 1);
    assert_eq!(result.d, OwnedTerm::Integer(42));
}

#[test]
fn test_struct_with_owned_term_field_map_payload() {
    let mut inner_map = BTreeMap::new();
    inner_map.insert(
        OwnedTerm::Binary(b"event".to_vec()),
        OwnedTerm::Binary(b"ready".to_vec()),
    );
    inner_map.insert(
        OwnedTerm::Binary(b"user_id".to_vec()),
        OwnedTerm::Integer(12345),
    );

    let payload = Payload {
        op: 2,
        d: OwnedTerm::Map(inner_map.clone()),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.op, 2);
    assert_eq!(result.d, OwnedTerm::Map(inner_map));
}

#[test]
fn test_struct_with_owned_term_field_list_payload() {
    let list = OwnedTerm::List(vec![
        OwnedTerm::Integer(1),
        OwnedTerm::Integer(2),
        OwnedTerm::Integer(3),
    ]);

    let payload = Payload {
        op: 3,
        d: list.clone(),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.op, 3);
    assert_eq!(result.d, list);
}

#[test]
fn test_struct_with_owned_term_field_nested_structure() {
    let mut inner = BTreeMap::new();
    inner.insert(
        OwnedTerm::Binary(b"nested".to_vec()),
        OwnedTerm::List(vec![OwnedTerm::Integer(1), OwnedTerm::Integer(2)]),
    );

    let payload = Payload {
        op: 4,
        d: OwnedTerm::Map(inner.clone()),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.op, 4);
    assert_eq!(result.d, OwnedTerm::Map(inner));
}

// Conditional deserialization based on op code

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ReadyEvent {
    user_id: i64,
    session_id: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MessageEvent {
    content: String,
    author: String,
}

#[test]
fn test_conditional_deserialization_based_on_op() {
    let ready_data = ReadyEvent {
        user_id: 123,
        session_id: "abc".to_string(),
    };
    let ready_term = to_term(&ready_data).unwrap();

    let payload = Payload {
        op: 1,
        d: ready_term,
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.op, 1);

    let ready: ReadyEvent = from_term(&result.d).unwrap();
    assert_eq!(ready.user_id, 123);
    assert_eq!(ready.session_id, "abc");
}

#[test]
fn test_conditional_deserialization_different_types() {
    let msg_data = MessageEvent {
        content: "Hello".to_string(),
        author: "Alice".to_string(),
    };
    let msg_term = to_term(&msg_data).unwrap();

    let payload = Payload { op: 2, d: msg_term };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.op, 2);

    let msg: MessageEvent = from_term(&result.d).unwrap();
    assert_eq!(msg.content, "Hello");
    assert_eq!(msg.author, "Alice");
}

// Direct OwnedTerm deserialization

#[test]
fn test_deserialize_bool_true_to_owned_term() {
    let term = to_term(&true).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(result, OwnedTerm::Atom(erltf::types::Atom::new("true")));
}

#[test]
fn test_deserialize_bool_false_to_owned_term() {
    let term = to_term(&false).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(result, OwnedTerm::Atom(erltf::types::Atom::new("false")));
}

#[test]
fn test_deserialize_integer_to_owned_term() {
    let term = to_term(&42i64).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(result, OwnedTerm::Integer(42));
}

#[test]
fn test_deserialize_negative_integer_to_owned_term() {
    let term = to_term(&(-999i64)).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(result, OwnedTerm::Integer(-999));
}

#[test]
fn test_deserialize_float_to_owned_term() {
    let term = to_term(&3.14f64).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(result, OwnedTerm::Float(3.14));
}

#[test]
fn test_deserialize_string_to_owned_term() {
    let term = to_term(&"hello").unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(result, OwnedTerm::Binary(b"hello".to_vec()));
}

#[test]
fn test_deserialize_bytes_to_owned_term() {
    // Vec<u8> serializes as list of integers, not binary.
    let bytes: Vec<u8> = vec![1, 2, 3, 4, 5];
    let term = to_term(&bytes).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(
        result,
        OwnedTerm::List(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::Integer(2),
            OwnedTerm::Integer(3),
            OwnedTerm::Integer(4),
            OwnedTerm::Integer(5),
        ])
    );
}

#[test]
fn test_deserialize_binary_to_owned_term() {
    let binary = OwnedTerm::Binary(vec![1, 2, 3, 4, 5]);
    let wrapper = Wrapper {
        data: binary.clone(),
    };
    let term = to_term(&wrapper).unwrap();
    let result: Wrapper = from_term(&term).unwrap();
    assert_eq!(result.data, binary);
}

#[test]
fn test_deserialize_vec_to_owned_term() {
    let vec = vec![1i64, 2, 3];
    let term = to_term(&vec).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(
        result,
        OwnedTerm::List(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::Integer(2),
            OwnedTerm::Integer(3),
        ])
    );
}

#[test]
fn test_deserialize_empty_vec_to_owned_term() {
    let vec: Vec<i64> = vec![];
    let term = to_term(&vec).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();
    assert_eq!(result, OwnedTerm::List(vec![]));
}

#[test]
fn test_deserialize_map_to_owned_term() {
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_string(), 42i64);

    let term = to_term(&map).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();

    let mut expected = BTreeMap::new();
    expected.insert(OwnedTerm::Binary(b"key".to_vec()), OwnedTerm::Integer(42));
    assert_eq!(result, OwnedTerm::Map(expected));
}

#[test]
fn test_deserialize_nested_map_to_owned_term() {
    let mut inner = std::collections::HashMap::new();
    inner.insert("nested_key".to_string(), 100i64);

    let mut outer = std::collections::HashMap::new();
    outer.insert("inner".to_string(), inner);

    let term = to_term(&outer).unwrap();
    let result: OwnedTerm = from_term(&term).unwrap();

    match result {
        OwnedTerm::Map(m) => {
            assert_eq!(m.len(), 1);
            let inner_term = m.get(&OwnedTerm::Binary(b"inner".to_vec())).unwrap();
            match inner_term {
                OwnedTerm::Map(inner_m) => {
                    assert_eq!(inner_m.len(), 1);
                    assert_eq!(
                        inner_m.get(&OwnedTerm::Binary(b"nested_key".to_vec())),
                        Some(&OwnedTerm::Integer(100))
                    );
                }
                _ => panic!("expected inner map"),
            }
        }
        _ => panic!("expected map"),
    }
}

// Vec<OwnedTerm> deserialization

#[test]
fn test_deserialize_to_vec_of_owned_terms() {
    let items = vec![1i64, 2, 3];
    let term = to_term(&items).unwrap();
    let result: Vec<OwnedTerm> = from_term(&term).unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0], OwnedTerm::Integer(1));
    assert_eq!(result[1], OwnedTerm::Integer(2));
    assert_eq!(result[2], OwnedTerm::Integer(3));
}

#[test]
fn test_deserialize_to_vec_of_owned_terms_mixed() {
    let items: Vec<OwnedTerm> = vec![
        OwnedTerm::Integer(42),
        OwnedTerm::Binary(b"test".to_vec()),
        OwnedTerm::Float(3.14),
    ];

    let original = MixedResult { items };
    let term = to_term(&original).unwrap();
    let result: MixedResult = from_term(&term).unwrap();

    assert_eq!(result.items.len(), 3);
    assert_eq!(result.items[0], OwnedTerm::Integer(42));
    assert_eq!(result.items[1], OwnedTerm::Binary(b"test".to_vec()));
    assert_eq!(result.items[2], OwnedTerm::Float(3.14));
}

// Option<OwnedTerm>

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct OptionalPayload {
    op: u16,
    d: Option<OwnedTerm>,
}

#[test]
fn test_struct_with_optional_owned_term_some() {
    let payload = OptionalPayload {
        op: 1,
        d: Some(OwnedTerm::Integer(42)),
    };

    let term = to_term(&payload).unwrap();
    let result: OptionalPayload = from_term(&term).unwrap();

    assert_eq!(result.op, 1);
    assert_eq!(result.d, Some(OwnedTerm::Integer(42)));
}

#[test]
fn test_struct_with_optional_owned_term_none() {
    let payload = OptionalPayload { op: 1, d: None };

    let term = to_term(&payload).unwrap();
    let result: OptionalPayload = from_term(&term).unwrap();

    assert_eq!(result.op, 1);
    assert_eq!(result.d, None);
}

// Edge cases

#[test]
fn test_deserialize_owned_term_preserves_structure() {
    let original = OwnedTerm::List(vec![
        OwnedTerm::Integer(1),
        OwnedTerm::List(vec![OwnedTerm::Integer(2), OwnedTerm::Integer(3)]),
        OwnedTerm::Integer(4),
    ]);

    let term = to_term(&Wrapper {
        data: original.clone(),
    })
    .unwrap();
    let result: Wrapper = from_term(&term).unwrap();

    assert_eq!(result.data, original);
}

#[test]
fn test_deeply_nested_structure() {
    let mut level3 = BTreeMap::new();
    level3.insert(OwnedTerm::Binary(b"value".to_vec()), OwnedTerm::Integer(42));

    let mut level2 = BTreeMap::new();
    level2.insert(
        OwnedTerm::Binary(b"level3".to_vec()),
        OwnedTerm::Map(level3),
    );

    let mut level1 = BTreeMap::new();
    level1.insert(
        OwnedTerm::Binary(b"level2".to_vec()),
        OwnedTerm::Map(level2),
    );

    let payload = Payload {
        op: 1,
        d: OwnedTerm::Map(level1.clone()),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Map(level1));
}

#[test]
fn test_empty_map_owned_term() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Map(BTreeMap::new()),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Map(BTreeMap::new()));
}

#[test]
fn test_empty_list_owned_term() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::List(vec![]),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::List(vec![]));
}

#[test]
fn test_large_integer_in_payload() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Integer(i64::MAX),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Integer(i64::MAX));
}

#[test]
fn test_nil_becomes_empty_list_on_roundtrip() {
    // Nil is Erlang's empty list terminator - semantically equivalent to [].
    // Through serde, Nil serializes as an empty sequence and deserializes as List([]).
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Nil,
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::List(vec![]));
}

#[test]
fn test_atom_becomes_binary_on_roundtrip() {
    // Non-boolean atoms serialize as strings, deserialize as Binary.
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Atom(erltf::types::Atom::new("my_custom_atom")),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Binary(b"my_custom_atom".to_vec()));
}

#[test]
fn test_empty_binary() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Binary(vec![]),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Binary(vec![]));
}

#[test]
fn test_negative_float() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Float(-123.456),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Float(-123.456));
}

#[test]
fn test_erlang_string_roundtrip() {
    // Erlang String type serializes as str, deserializes as Binary.
    let payload = Payload {
        op: 1,
        d: OwnedTerm::String("hello".to_string()),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Binary(b"hello".to_vec()));
}

#[test]
fn test_tuple_becomes_list_on_roundtrip() {
    // Tuples serialize as sequences, deserialize as Lists.
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Tuple(vec![OwnedTerm::Integer(1), OwnedTerm::Integer(2)]),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(
        result.d,
        OwnedTerm::List(vec![OwnedTerm::Integer(1), OwnedTerm::Integer(2)])
    );
}

#[test]
fn test_improper_list_becomes_list_on_roundtrip() {
    // ImproperList flattens to regular List through serde.
    let payload = Payload {
        op: 1,
        d: OwnedTerm::ImproperList {
            elements: vec![OwnedTerm::Integer(1), OwnedTerm::Integer(2)],
            tail: Box::new(OwnedTerm::Integer(3)),
        },
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(
        result.d,
        OwnedTerm::List(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::Integer(2),
            OwnedTerm::Integer(3)
        ])
    );
}

#[test]
fn test_bit_binary_becomes_binary_on_roundtrip() {
    // BitBinary loses bit count, becomes regular Binary.
    let payload = Payload {
        op: 1,
        d: OwnedTerm::BitBinary {
            bytes: vec![0xFF, 0x0F],
            bits: 4,
        },
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Binary(vec![0xFF, 0x0F]));
}

#[test]
fn test_small_bigint_becomes_integer() {
    // Small BigInt values fit in i64 and become Integer.
    let big = erltf::types::BigInt::new(false, vec![42, 0, 0, 0, 0, 0, 0, 0]);
    let payload = Payload {
        op: 1,
        d: OwnedTerm::BigInt(big),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Integer(42));
}

#[test]
fn test_negative_small_bigint_becomes_integer() {
    let big = erltf::types::BigInt::new(true, vec![42, 0, 0, 0, 0, 0, 0, 0]);
    let payload = Payload {
        op: 1,
        d: OwnedTerm::BigInt(big),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Integer(-42));
}

#[test]
fn test_i64_min_boundary() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Integer(i64::MIN),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Integer(i64::MIN));
}

#[test]
fn test_u64_max_becomes_bigint() {
    // u64::MAX exceeds i64::MAX, so it becomes BigInt.
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct U64Payload {
        val: u64,
    }

    let payload = U64Payload { val: u64::MAX };
    let term = to_term(&payload).unwrap();

    // The result deserializes the BigInt back correctly.
    let result: U64Payload = from_term(&term).unwrap();
    assert_eq!(result.val, u64::MAX);
}

#[test]
fn test_nil_atom_roundtrip() {
    // Atom("nil") serializes as None, becomes Atom("nil") or Atom("undefined") on roundtrip.
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Atom(erltf::types::Atom::new("nil")),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    // Without elixir-interop, None deserializes to Atom("undefined").
    #[cfg(not(feature = "elixir-interop"))]
    assert_eq!(
        result.d,
        OwnedTerm::Atom(erltf::types::Atom::new("undefined"))
    );
    #[cfg(feature = "elixir-interop")]
    assert_eq!(result.d, OwnedTerm::Atom(erltf::types::Atom::new("nil")));
}

#[test]
fn test_undefined_atom_roundtrip() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Atom(erltf::types::Atom::new("undefined")),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    #[cfg(not(feature = "elixir-interop"))]
    assert_eq!(
        result.d,
        OwnedTerm::Atom(erltf::types::Atom::new("undefined"))
    );
    #[cfg(feature = "elixir-interop")]
    assert_eq!(result.d, OwnedTerm::Atom(erltf::types::Atom::new("nil")));
}

#[test]
fn test_zero_float() {
    let payload = Payload {
        op: 1,
        d: OwnedTerm::Float(0.0),
    };

    let term = to_term(&payload).unwrap();
    let result: Payload = from_term(&term).unwrap();

    assert_eq!(result.d, OwnedTerm::Float(0.0));
}
