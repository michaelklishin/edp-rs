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
use erltf::types::Atom;
use erltf::{decode, encode};
use proptest::prelude::*;

fn arb_atom() -> impl Strategy<Value = Atom> {
    "[a-z][a-z0-9_]{0,20}".prop_map(Atom::new)
}

fn arb_simple_term() -> impl Strategy<Value = OwnedTerm> {
    prop_oneof![
        any::<u8>().prop_map(|v| OwnedTerm::Integer(v as i64)),
        any::<i32>().prop_map(|v| OwnedTerm::Integer(v as i64)),
        any::<f64>().prop_map(OwnedTerm::Float),
        arb_atom().prop_map(OwnedTerm::Atom),
        prop::collection::vec(any::<u8>(), 0..100).prop_map(OwnedTerm::Binary),
    ]
}

fn arb_term() -> impl Strategy<Value = OwnedTerm> {
    arb_simple_term().prop_recursive(3, 32, 10, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 1..10).prop_map(OwnedTerm::List),
            prop::collection::vec(inner.clone(), 0..10).prop_map(OwnedTerm::Tuple),
        ]
    })
}

proptest! {
    #[test]
    fn test_prop_roundtrip_integer(value in any::<i32>()) {
        let term = OwnedTerm::Integer(value as i64);
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(term, decoded);
    }

    #[test]
    fn test_prop_roundtrip_float(value in any::<f64>()) {
        if value.is_finite() {
            let term = OwnedTerm::Float(value);
            let encoded = encode(&term).unwrap();
            let decoded = decode(&encoded).unwrap();
            prop_assert_eq!(term, decoded);
        }
    }

    #[test]
    fn test_prop_roundtrip_atom(name in "[a-z][a-z0-9_]{0,50}") {
        let term = OwnedTerm::Atom(Atom::new(name));
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(term, decoded);
    }

    #[test]
    fn test_prop_roundtrip_binary(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let term = OwnedTerm::Binary(data);
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(term, decoded);
    }

    #[test]
    fn test_prop_roundtrip_string(s in "[a-zA-Z0-9 ]{0,100}") {
        let term = OwnedTerm::String(s.clone());
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(OwnedTerm::Binary(s.into_bytes()), decoded);
    }

    #[test]
    fn test_prop_roundtrip_list(elements in prop::collection::vec(any::<i32>(), 1..50)) {
        let term = OwnedTerm::List(
            elements.into_iter().map(|i| OwnedTerm::Integer(i as i64)).collect()
        );
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(term, decoded);
    }

    #[test]
    fn test_prop_roundtrip_tuple(elements in prop::collection::vec(any::<i32>(), 0..50)) {
        let term = OwnedTerm::Tuple(
            elements.into_iter().map(|i| OwnedTerm::Integer(i as i64)).collect()
        );
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(term, decoded);
    }

    #[test]
    fn test_prop_roundtrip_term(term in arb_term()) {
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(term, decoded);
    }

    #[test]
    fn test_prop_encode_is_deterministic(term in arb_term()) {
        let encoded1 = encode(&term).unwrap();
        let encoded2 = encode(&term).unwrap();
        prop_assert_eq!(encoded1, encoded2);
    }

    #[test]
    fn test_prop_atom_caching_correctness(name in "[a-z][a-z0-9_]{0,50}") {
        let atom1 = Atom::new(&name);
        let atom2 = Atom::new(&name);
        prop_assert_eq!(atom1, atom2);
    }

    #[test]
    fn test_prop_common_atoms_cached(name in prop_oneof![Just("ok"), Just("error"), Just("true"), Just("false")]) {
        let atom1 = Atom::new(name);
        let atom2 = Atom::new(name);
        prop_assert!(std::sync::Arc::ptr_eq(&atom1.name, &atom2.name));
    }

    #[test]
    fn test_prop_map_roundtrip(entries in prop::collection::vec((any::<u8>(), any::<i32>()), 0..20)) {
        let map: std::collections::BTreeMap<_, _> = entries
            .into_iter()
            .map(|(k, v)| (OwnedTerm::Integer(k as i64), OwnedTerm::Integer(v as i64)))
            .collect();
        let term = OwnedTerm::Map(map);
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(term, decoded);
    }
}
