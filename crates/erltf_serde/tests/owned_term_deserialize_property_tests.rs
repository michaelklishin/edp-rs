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
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Payload {
    op: u16,
    d: OwnedTerm,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NestedPayload {
    op: u16,
    d: OwnedTerm,
    extra: Option<OwnedTerm>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct VecWrapper {
    items: Vec<OwnedTerm>,
}

// Nil excluded: semantically equivalent to [] in Erlang, becomes List([]) through serde.
fn arb_simple_owned_term() -> impl Strategy<Value = OwnedTerm> {
    prop_oneof![
        any::<i64>().prop_map(OwnedTerm::Integer),
        any::<f64>()
            .prop_filter("finite floats only", |f| f.is_finite())
            .prop_map(OwnedTerm::Float),
        "[a-zA-Z0-9_]{1,20}".prop_map(|s| OwnedTerm::Binary(s.into_bytes())),
        Just(OwnedTerm::Atom(erltf::types::Atom::new("true"))),
        Just(OwnedTerm::Atom(erltf::types::Atom::new("false"))),
    ]
}

fn arb_owned_term() -> impl Strategy<Value = OwnedTerm> {
    arb_simple_owned_term().prop_recursive(3, 10, 5, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..5).prop_map(OwnedTerm::List),
            prop::collection::btree_map(inner.clone(), inner.clone(), 0..3)
                .prop_map(OwnedTerm::Map),
        ]
    })
}

proptest! {
    #[test]
    fn prop_roundtrip_integer_payload(op in any::<u16>(), val in any::<i64>()) {
        let payload = Payload {
            op,
            d: OwnedTerm::Integer(val),
        };
        let term = to_term(&payload).unwrap();
        let result: Payload = from_term(&term).unwrap();
        prop_assert_eq!(payload, result);
    }

    #[test]
    fn prop_roundtrip_float_payload(
        op in any::<u16>(),
        val in any::<f64>().prop_filter("finite floats only", |f| f.is_finite())
    ) {
        let payload = Payload {
            op,
            d: OwnedTerm::Float(val),
        };
        let term = to_term(&payload).unwrap();
        let result: Payload = from_term(&term).unwrap();
        prop_assert_eq!(payload, result);
    }

    #[test]
    fn prop_roundtrip_binary_payload(op in any::<u16>(), data in prop::collection::vec(any::<u8>(), 0..100)) {
        let payload = Payload {
            op,
            d: OwnedTerm::Binary(data),
        };
        let term = to_term(&payload).unwrap();
        let result: Payload = from_term(&term).unwrap();
        prop_assert_eq!(payload, result);
    }

    #[test]
    fn prop_roundtrip_list_payload(
        op in any::<u16>(),
        items in prop::collection::vec(any::<i64>(), 0..10)
    ) {
        let list = items.into_iter().map(OwnedTerm::Integer).collect();
        let payload = Payload {
            op,
            d: OwnedTerm::List(list),
        };
        let term = to_term(&payload).unwrap();
        let result: Payload = from_term(&term).unwrap();
        prop_assert_eq!(payload, result);
    }

    #[test]
    fn prop_roundtrip_map_payload(
        op in any::<u16>(),
        keys in prop::collection::vec("[a-z]{1,10}", 0..5),
        values in prop::collection::vec(any::<i64>(), 0..5)
    ) {
        let mut map = BTreeMap::new();
        for (k, v) in keys.into_iter().zip(values.into_iter()) {
            map.insert(OwnedTerm::Binary(k.into_bytes()), OwnedTerm::Integer(v));
        }
        let payload = Payload {
            op,
            d: OwnedTerm::Map(map),
        };
        let term = to_term(&payload).unwrap();
        let result: Payload = from_term(&term).unwrap();
        prop_assert_eq!(payload, result);
    }

    #[test]
    fn prop_roundtrip_arbitrary_owned_term(d in arb_owned_term()) {
        let payload = Payload { op: 1, d: d.clone() };
        let term = to_term(&payload).unwrap();
        let result: Payload = from_term(&term).unwrap();
        prop_assert_eq!(payload.d, result.d);
    }

    #[test]
    fn prop_roundtrip_nested_payload_with_optional(
        op in any::<u16>(),
        d in arb_simple_owned_term(),
        extra in proptest::option::of(arb_simple_owned_term())
    ) {
        let payload = NestedPayload { op, d, extra };
        let term = to_term(&payload).unwrap();
        let result: NestedPayload = from_term(&term).unwrap();
        prop_assert_eq!(payload, result);
    }

    #[test]
    fn prop_vec_of_owned_terms_roundtrip(items in prop::collection::vec(arb_simple_owned_term(), 0..10)) {
        let wrapper = VecWrapper { items };
        let term = to_term(&wrapper).unwrap();
        let result: VecWrapper = from_term(&term).unwrap();
        prop_assert_eq!(wrapper, result);
    }

    #[test]
    fn prop_deeply_nested_maps(depth in 1usize..4, key in "[a-z]{1,5}", val in any::<i64>()) {
        let mut current = OwnedTerm::Integer(val);
        for _ in 0..depth {
            let mut map = BTreeMap::new();
            map.insert(OwnedTerm::Binary(key.as_bytes().to_vec()), current);
            current = OwnedTerm::Map(map);
        }

        let payload = Payload { op: 1, d: current };
        let term = to_term(&payload).unwrap();
        let result: Payload = from_term(&term).unwrap();
        prop_assert_eq!(payload, result);
    }

    #[test]
    fn prop_deeply_nested_lists(depth in 1usize..4, val in any::<i64>()) {
        let mut current = OwnedTerm::Integer(val);
        for _ in 0..depth {
            current = OwnedTerm::List(vec![current]);
        }

        let payload = Payload { op: 1, d: current };
        let term = to_term(&payload).unwrap();
        let result: Payload = from_term(&term).unwrap();
        prop_assert_eq!(payload, result);
    }
}
