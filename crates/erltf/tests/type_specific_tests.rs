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
use erltf::types::{Atom, BigInt, ExternalFun, ExternalPid};
use erltf::{decode, encode};
use std::cmp::Ordering;

// ============================================================================
// PID Parsing Tests
// ============================================================================

#[test]
fn test_parse_valid_pid() {
    let node = Atom::new("rabbit@localhost");
    let pid = ExternalPid::from_string(node.clone(), "<0.208.0>").unwrap();

    assert_eq!(pid.id, 0);
    assert_eq!(pid.serial, 208);
    assert_eq!(pid.creation, 0);
    assert_eq!(pid.node, node);
}

#[test]
fn test_parse_pid_with_larger_numbers() {
    let node = Atom::new("test@server");
    let pid = ExternalPid::from_string(node.clone(), "<12345.67890.4>").unwrap();

    assert_eq!(pid.id, 12345);
    assert_eq!(pid.serial, 67890);
    assert_eq!(pid.creation, 4);
}

#[test]
fn test_parse_pid_with_whitespace() {
    let node = Atom::new("test@localhost");
    let pid = ExternalPid::from_string(node, "  <1.2.3>  ").unwrap();

    assert_eq!(pid.id, 1);
    assert_eq!(pid.serial, 2);
    assert_eq!(pid.creation, 3);
}

#[test]
fn test_parse_invalid_pid_missing_brackets() {
    let node = Atom::new("test@localhost");
    let result = ExternalPid::from_string(node, "1.2.3");

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        erltf::errors::DecodeError::InvalidPidFormat(_)
    ));
    assert!(err.to_string().contains("format <id.serial.creation>"));
}

#[test]
fn test_parse_invalid_pid_wrong_part_count() {
    let node = Atom::new("test@localhost");
    let result = ExternalPid::from_string(node, "<1.2>");

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        erltf::errors::DecodeError::InvalidPidFormat(_)
    ));
    assert!(err.to_string().contains("3 parts"));
}

#[test]
fn test_parse_invalid_pid_non_numeric() {
    let node = Atom::new("test@localhost");
    let result = ExternalPid::from_string(node, "<a.b.c>");

    assert!(result.is_err());
}

#[test]
fn test_format_pid_to_string() {
    let node = Atom::new("test@localhost");
    let pid = ExternalPid::new(node, 123, 456, 7);

    assert_eq!(format!("{}", pid), "<123.456.7>");
}

#[test]
fn test_roundtrip_pid_parsing() {
    let node = Atom::new("rabbit@server");
    let original = ExternalPid::new(node.clone(), 999, 1234, 5);

    let formatted = format!("{}", original);
    let parsed = ExternalPid::from_string(node, &formatted).unwrap();

    assert_eq!(original, parsed);
}

// ============================================================================
// ExternalFun Tests
// ============================================================================

#[test]
fn test_external_fun_roundtrip() {
    let fun = OwnedTerm::ExternalFun(ExternalFun::new(Atom::new("lists"), Atom::new("sort"), 1));

    let encoded = encode(&fun).unwrap();
    let decoded = decode(&encoded).unwrap();

    assert_eq!(fun, decoded);
}

#[test]
fn test_external_fun_ordering() {
    let fun1 = OwnedTerm::ExternalFun(ExternalFun::new(Atom::new("lists"), Atom::new("map"), 2));

    let fun2 = OwnedTerm::ExternalFun(ExternalFun::new(Atom::new("lists"), Atom::new("sort"), 1));

    assert!(fun1 < fun2);
}

#[test]
fn test_fun_type_ordering() {
    let ref_term = OwnedTerm::Reference(erltf::types::ExternalReference::new(
        Atom::new("node@host"),
        1,
        vec![1, 2, 3],
    ));

    let fun = OwnedTerm::ExternalFun(ExternalFun::new(Atom::new("lists"), Atom::new("sort"), 1));

    let port = OwnedTerm::Port(erltf::types::ExternalPort::new(
        Atom::new("node@host"),
        1,
        1,
    ));

    assert!(ref_term < fun);
    assert!(fun < port);
}

#[test]
fn test_decode_erlang_external_fun() {
    let erlang_bytes = vec![
        131, 113, 119, 5, 108, 105, 115, 116, 115, 119, 4, 115, 111, 114, 116, 97, 1,
    ];

    let decoded = decode(&erlang_bytes).unwrap();

    match &decoded {
        OwnedTerm::ExternalFun(fun) => {
            assert_eq!(fun.module, "lists");
            assert_eq!(fun.function, "sort");
            assert_eq!(fun.arity, 1);
        }
        _ => panic!("Expected ExternalFun, got {:?}", decoded),
    }

    let re_encoded = encode(&decoded).unwrap();
    let re_decoded = decode(&re_encoded).unwrap();
    assert_eq!(decoded, re_decoded);
}

// ============================================================================
// BigInt Conversion Tests
// ============================================================================

#[test]
fn test_bigint_float_comparison_with_known_values() {
    let bigint_2_pow_63 = OwnedTerm::BigInt(BigInt::new(false, vec![0, 0, 0, 0, 0, 0, 0, 128]));
    let float_2_pow_63 = OwnedTerm::float(9223372036854775808.0);

    assert_eq!(bigint_2_pow_63.cmp(&float_2_pow_63), Ordering::Equal);
}

#[test]
fn test_bigint_float_less_than() {
    let bigint_256 = OwnedTerm::BigInt(BigInt::new(false, vec![0, 1]));
    let float_1000 = OwnedTerm::float(1000.0);

    assert!(bigint_256 < float_1000);
}

#[test]
fn test_bigint_float_greater_than() {
    let bigint_large = OwnedTerm::BigInt(BigInt::new(false, vec![0, 0, 1]));
    let float_100 = OwnedTerm::float(100.0);

    assert!(bigint_large > float_100);
}

#[test]
fn test_negative_bigint_float_comparison() {
    let neg_bigint = OwnedTerm::BigInt(BigInt::new(true, vec![0, 1]));
    let neg_float = OwnedTerm::float(-256.0);

    assert_eq!(neg_bigint.cmp(&neg_float), Ordering::Equal);
}

#[test]
fn test_small_bigint_values() {
    let bigint_1 = OwnedTerm::BigInt(BigInt::new(false, vec![1]));
    let bigint_255 = OwnedTerm::BigInt(BigInt::new(false, vec![255]));
    let bigint_256 = OwnedTerm::BigInt(BigInt::new(false, vec![0, 1]));

    assert_eq!(bigint_1.cmp(&OwnedTerm::float(1.0)), Ordering::Equal);
    assert_eq!(bigint_255.cmp(&OwnedTerm::float(255.0)), Ordering::Equal);
    assert_eq!(bigint_256.cmp(&OwnedTerm::float(256.0)), Ordering::Equal);
}

#[test]
fn test_bigint_byte_order() {
    let bigint_le = OwnedTerm::BigInt(BigInt::new(false, vec![1, 2, 3]));
    let value = 1.0 + 2.0 * 256.0 + 3.0 * 256.0 * 256.0;
    let float_val = OwnedTerm::float(value);

    assert_eq!(bigint_le.cmp(&float_val), Ordering::Equal);
}

#[test]
fn test_zero_bigint() {
    let zero_bigint = OwnedTerm::BigInt(BigInt::new(false, vec![]));
    let zero_float = OwnedTerm::float(0.0);

    assert_eq!(zero_bigint.cmp(&zero_float), Ordering::Equal);
}

#[test]
fn test_negative_zero_bigint() {
    let neg_zero_bigint = OwnedTerm::BigInt(BigInt::new(true, vec![]));
    let zero_float = OwnedTerm::float(0.0);

    assert_eq!(neg_zero_bigint.cmp(&zero_float), Ordering::Equal);
}

// ============================================================================
// Erlang Term Value Ordering Tests
// ============================================================================

#[test]
fn test_number_less_than_atom() {
    assert!(OwnedTerm::Integer(42) < OwnedTerm::atom("hello"));
    assert!(OwnedTerm::float(2.5) < OwnedTerm::atom("world"));
}

#[test]
fn test_atom_less_than_reference() {
    let atom = OwnedTerm::atom("test");
    let reference = OwnedTerm::Reference(erltf::types::ExternalReference::new(
        Atom::new("node@host"),
        1,
        vec![1, 2, 3],
    ));
    assert!(atom < reference);
}

#[test]
fn test_reference_less_than_fun() {
    let reference = OwnedTerm::Reference(erltf::types::ExternalReference::new(
        Atom::new("node@host"),
        1,
        vec![1, 2, 3],
    ));
    let fun = OwnedTerm::ExternalFun(ExternalFun::new(Atom::new("lists"), Atom::new("sort"), 1));
    assert!(reference < fun);
}

#[test]
fn test_fun_less_than_port() {
    let fun = OwnedTerm::ExternalFun(ExternalFun::new(Atom::new("lists"), Atom::new("sort"), 1));
    let port = OwnedTerm::Port(erltf::types::ExternalPort::new(
        Atom::new("node@host"),
        1,
        1,
    ));
    assert!(fun < port);
}

#[test]
fn test_port_less_than_pid() {
    let port = OwnedTerm::Port(erltf::types::ExternalPort::new(
        Atom::new("node@host"),
        1,
        1,
    ));
    let pid = OwnedTerm::Pid(ExternalPid::new(Atom::new("node@host"), 1, 2, 3));
    assert!(port < pid);
}

#[test]
fn test_pid_less_than_tuple() {
    let pid = OwnedTerm::Pid(ExternalPid::new(Atom::new("node@host"), 1, 2, 3));
    let tuple = OwnedTerm::tuple(vec![OwnedTerm::Integer(1)]);
    assert!(pid < tuple);
}

#[test]
fn test_tuple_less_than_map() {
    let tuple = OwnedTerm::tuple(vec![OwnedTerm::Integer(1)]);
    let mut map = std::collections::BTreeMap::new();
    map.insert(OwnedTerm::atom("key"), OwnedTerm::Integer(1));
    let map_term = OwnedTerm::Map(map);
    assert!(tuple < map_term);
}

#[test]
fn test_map_less_than_list() {
    let mut map = std::collections::BTreeMap::new();
    map.insert(OwnedTerm::atom("key"), OwnedTerm::Integer(1));
    let map_term = OwnedTerm::Map(map);
    let list = OwnedTerm::list(vec![OwnedTerm::Integer(1)]);
    assert!(map_term < list);
}

#[test]
fn test_list_less_than_binary() {
    let list = OwnedTerm::list(vec![OwnedTerm::Integer(1)]);
    let binary = OwnedTerm::Binary(vec![1, 2, 3]);
    assert!(list < binary);
}

#[test]
fn test_complete_type_ordering_chain() {
    let terms = vec![
        OwnedTerm::Integer(42),
        OwnedTerm::atom("atom"),
        OwnedTerm::Reference(erltf::types::ExternalReference::new(
            Atom::new("node@host"),
            1,
            vec![1],
        )),
        OwnedTerm::ExternalFun(ExternalFun::new(Atom::new("mod"), Atom::new("fun"), 0)),
        OwnedTerm::Port(erltf::types::ExternalPort::new(
            Atom::new("node@host"),
            1,
            1,
        )),
        OwnedTerm::Pid(ExternalPid::new(Atom::new("node@host"), 1, 2, 3)),
        OwnedTerm::tuple(vec![]),
        OwnedTerm::Map(std::collections::BTreeMap::new()),
        OwnedTerm::list(vec![]),
        OwnedTerm::Binary(vec![]),
    ];

    for i in 0..terms.len() - 1 {
        assert!(
            terms[i] < terms[i + 1],
            "Expected {:?} < {:?}",
            terms[i],
            terms[i + 1]
        );
    }
}
