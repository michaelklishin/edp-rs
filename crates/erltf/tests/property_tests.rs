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

use erltf::types::{
    Atom, BigInt, ExternalFun, ExternalPid, ExternalPort, ExternalReference, InternalFun, Sign,
};
use erltf::{OwnedTerm, decode, encode};
use proptest::prelude::*;

fn arb_atom() -> impl Strategy<Value = Atom> {
    "[a-z][a-z0-9_]{0,20}".prop_map(Atom::new)
}

fn arb_node_atom() -> impl Strategy<Value = Atom> {
    "[a-z][a-z0-9_]{0,10}@[a-z][a-z0-9]{0,10}".prop_map(Atom::new)
}

fn arb_pid() -> impl Strategy<Value = ExternalPid> {
    (arb_node_atom(), any::<u32>(), any::<u32>(), any::<u32>())
        .prop_map(|(node, id, serial, creation)| ExternalPid::new(node, id, serial, creation))
}

fn arb_port() -> impl Strategy<Value = ExternalPort> {
    (arb_node_atom(), any::<u64>(), any::<u32>())
        .prop_map(|(node, id, creation)| ExternalPort::new(node, id, creation))
}

fn arb_reference() -> impl Strategy<Value = ExternalReference> {
    (
        arb_node_atom(),
        any::<u32>(),
        prop::collection::vec(any::<u32>(), 1..5),
    )
        .prop_map(|(node, creation, ids)| ExternalReference::new(node, creation, ids))
}

fn arb_improper_list_tail() -> impl Strategy<Value = OwnedTerm> {
    prop_oneof![
        any::<i32>().prop_map(|v| OwnedTerm::Integer(v as i64)),
        arb_atom().prop_map(OwnedTerm::Atom),
        prop::collection::vec(any::<u8>(), 0..50).prop_map(OwnedTerm::Binary),
    ]
}

fn arb_improper_list() -> impl Strategy<Value = OwnedTerm> {
    (
        prop::collection::vec(any::<i32>(), 1..10),
        arb_improper_list_tail(),
    )
        .prop_map(|(elements, tail)| OwnedTerm::ImproperList {
            elements: elements
                .into_iter()
                .map(|i| OwnedTerm::Integer(i as i64))
                .collect(),
            tail: Box::new(tail),
        })
}

fn arb_bit_binary() -> impl Strategy<Value = OwnedTerm> {
    (prop::collection::vec(any::<u8>(), 1..100), 1u8..8u8)
        .prop_map(|(bytes, bits)| OwnedTerm::BitBinary { bytes, bits })
}

fn arb_bigint() -> impl Strategy<Value = BigInt> {
    (prop::bool::ANY, prop::collection::vec(any::<u8>(), 1..50)).prop_map(|(positive, digits)| {
        let sign = if positive {
            Sign::Positive
        } else {
            Sign::Negative
        };
        BigInt::new(sign, digits)
    })
}

fn arb_external_fun() -> impl Strategy<Value = ExternalFun> {
    (arb_atom(), arb_atom(), any::<u8>())
        .prop_map(|(module, function, arity)| ExternalFun::new(module, function, arity))
}

fn arb_free_var() -> impl Strategy<Value = OwnedTerm> {
    prop_oneof![
        any::<i32>().prop_map(|v| OwnedTerm::Integer(v as i64)),
        arb_atom().prop_map(OwnedTerm::Atom),
        prop::collection::vec(any::<u8>(), 0..20).prop_map(OwnedTerm::Binary),
    ]
}

fn arb_internal_fun() -> impl Strategy<Value = InternalFun> {
    (
        any::<u8>(),
        prop::array::uniform16(any::<u8>()),
        any::<u32>(),
        0u32..5u32,
        arb_atom(),
        // old_index and old_uniq must fit in i32 for INTEGER_EXT encoding
        0u32..(i32::MAX as u32),
        0u32..(i32::MAX as u32),
        arb_pid(),
    )
        .prop_flat_map(
            |(arity, uniq, index, num_free, module, old_index, old_uniq, pid)| {
                prop::collection::vec(arb_free_var(), num_free as usize).prop_map(
                    move |free_vars| {
                        InternalFun::new(
                            arity,
                            uniq,
                            index,
                            num_free,
                            module.clone(),
                            old_index,
                            old_uniq,
                            pid.clone(),
                            free_vars,
                        )
                    },
                )
            },
        )
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
    fn test_prop_roundtrip_pid(pid in arb_pid()) {
        let term = OwnedTerm::Pid(pid);
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        let re_encoded = encode(&decoded).unwrap();
        prop_assert_eq!(term, decoded);
        prop_assert_eq!(encoded, re_encoded, "byte-level round-trip must be identical");
    }

    #[test]
    fn test_prop_roundtrip_port(port in arb_port()) {
        let term = OwnedTerm::Port(port);
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        let re_encoded = encode(&decoded).unwrap();
        prop_assert_eq!(term, decoded);
        prop_assert_eq!(encoded, re_encoded, "byte-level round-trip must be identical");
    }

    #[test]
    fn test_prop_roundtrip_reference(reference in arb_reference()) {
        let term = OwnedTerm::Reference(reference);
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        let re_encoded = encode(&decoded).unwrap();
        prop_assert_eq!(term, decoded);
        prop_assert_eq!(encoded, re_encoded, "byte-level round-trip must be identical");
    }

    #[test]
    fn test_prop_roundtrip_improper_list(term in arb_improper_list()) {
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        let re_encoded = encode(&decoded).unwrap();
        prop_assert_eq!(term, decoded);
        prop_assert_eq!(encoded, re_encoded, "byte-level round-trip must be identical");
    }

    #[test]
    fn test_prop_roundtrip_bit_binary(term in arb_bit_binary()) {
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        let re_encoded = encode(&decoded).unwrap();
        prop_assert_eq!(term, decoded);
        prop_assert_eq!(encoded, re_encoded, "byte-level round-trip must be identical");
    }

    #[test]
    fn test_prop_roundtrip_bigint(bigint in arb_bigint()) {
        let term = OwnedTerm::BigInt(bigint);
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        let re_encoded = encode(&decoded).unwrap();
        prop_assert_eq!(term, decoded);
        prop_assert_eq!(encoded, re_encoded, "byte-level round-trip must be identical");
    }

    #[test]
    fn test_prop_roundtrip_external_fun(fun in arb_external_fun()) {
        let term = OwnedTerm::ExternalFun(fun);
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        let re_encoded = encode(&decoded).unwrap();
        prop_assert_eq!(term, decoded);
        prop_assert_eq!(encoded, re_encoded, "byte-level round-trip must be identical");
    }

    #[test]
    fn test_prop_roundtrip_internal_fun(fun in arb_internal_fun()) {
        let term = OwnedTerm::InternalFun(Box::new(fun));
        let encoded = encode(&term).unwrap();
        let decoded = decode(&encoded).unwrap();
        let re_encoded = encode(&decoded).unwrap();
        prop_assert_eq!(term, decoded);
        prop_assert_eq!(encoded, re_encoded, "byte-level round-trip must be identical");
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

    #[test]
    fn test_prop_charlist_roundtrip(s in "[a-zA-Z0-9 ]{0,50}") {
        let charlist = OwnedTerm::charlist(&s);
        let back = charlist.as_charlist_string();
        prop_assert_eq!(back, Some(s));
    }

    #[test]
    fn test_prop_charlist_is_charlist(s in "[a-zA-Z0-9 ]{0,50}") {
        let charlist = OwnedTerm::charlist(&s);
        prop_assert!(charlist.is_charlist());
    }

    #[test]
    fn test_prop_as_list_or_empty_preserves_list(elements in prop::collection::vec(any::<i32>(), 0..20)) {
        let list = OwnedTerm::List(
            elements.iter().map(|i| OwnedTerm::Integer(*i as i64)).collect()
        );
        prop_assert_eq!(list.as_list_or_empty().len(), elements.len());
    }

    #[test]
    fn test_prop_format_as_mfa_consistent(
        module in "[a-z][a-z0-9_]{0,20}",
        function in "[a-z][a-z0-9_]{0,20}",
        arity in 0u8..255u8
    ) {
        let term = OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new(&module)),
            OwnedTerm::Atom(Atom::new(&function)),
            OwnedTerm::Integer(arity as i64),
        ]);
        let formatted = term.format_as_mfa().unwrap();
        prop_assert_eq!(formatted, format!("{}:{}/{}", module, function, arity));
    }

    #[test]
    fn test_prop_proplist_get_i64_or_default(value in any::<i64>(), default in any::<i64>()) {
        let proplist = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("key")),
            OwnedTerm::Integer(value),
        ])]);
        prop_assert_eq!(proplist.proplist_get_i64_or("key", default), value);
        prop_assert_eq!(proplist.proplist_get_i64_or("missing", default), default);
    }

    #[test]
    fn test_prop_proplist_get_bool_or_default(value: bool, default: bool) {
        let proplist = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("key")),
            OwnedTerm::Atom(Atom::new(if value { "true" } else { "false" })),
        ])]);
        prop_assert_eq!(proplist.proplist_get_bool_or("key", default), value);
        prop_assert_eq!(proplist.proplist_get_bool_or("missing", default), default);
    }

    #[test]
    fn test_prop_as_erlang_string_or(s in "[a-zA-Z0-9 ]{0,50}", default in "[a-z]{0,10}") {
        let charlist = OwnedTerm::charlist(&s);
        prop_assert_eq!(charlist.as_erlang_string_or(&default), s);
        prop_assert_eq!(OwnedTerm::Integer(42).as_erlang_string_or(&default), default);
    }

    #[test]
    fn test_prop_tuple_get_returns_correct_element(elements in prop::collection::vec(any::<i32>(), 1..10)) {
        let tuple = OwnedTerm::Tuple(
            elements.iter().map(|i| OwnedTerm::Integer(*i as i64)).collect()
        );
        for (idx, expected) in elements.iter().enumerate() {
            let got = tuple.tuple_get(idx);
            prop_assert!(got.is_some());
            prop_assert_eq!(got.unwrap(), &OwnedTerm::Integer(*expected as i64));
        }
        prop_assert!(tuple.tuple_get(elements.len()).is_none());
    }

    #[test]
    fn test_prop_tuple_get_string_charlist(
        s0 in "[a-z]+",
        s1 in "[a-zA-Z0-9 ]{0,30}",
        s2 in "[0-9\\.]{1,10}"
    ) {
        let tuple = OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new(&s0)),
            OwnedTerm::charlist(&s1),
            OwnedTerm::charlist(&s2),
        ]);
        prop_assert_eq!(tuple.tuple_get_atom_string(0), Some(s0));
        prop_assert_eq!(tuple.tuple_get_string(1), Some(s1));
        prop_assert_eq!(tuple.tuple_get_string(2), Some(s2));
    }
}
