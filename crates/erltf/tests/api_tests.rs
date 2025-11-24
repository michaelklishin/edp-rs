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
use erltf::types::{Atom, ExternalReference};
use erltf::{decode, encode};
use std::collections::BTreeMap;

// ============================================================================
// Ergonomics Tests
// ============================================================================

#[test]
fn test_boolean_constructors() {
    let true_term = OwnedTerm::boolean(true);
    let false_term = OwnedTerm::boolean(false);

    assert_eq!(true_term, OwnedTerm::atom("true"));
    assert_eq!(false_term, OwnedTerm::atom("false"));
}

#[test]
fn test_ok_error_constructors() {
    let ok = OwnedTerm::ok();
    let error = OwnedTerm::error();

    assert_eq!(ok, OwnedTerm::atom("ok"));
    assert_eq!(error, OwnedTerm::atom("error"));
}

#[test]
fn test_ok_tuple() {
    let result = OwnedTerm::ok_tuple(OwnedTerm::integer(42));
    let expected = OwnedTerm::tuple(vec![OwnedTerm::ok(), OwnedTerm::integer(42)]);

    assert_eq!(result, expected);

    let encoded = encode(&result).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(result, decoded);
}

#[test]
fn test_error_tuple() {
    let result = OwnedTerm::error_tuple(OwnedTerm::atom("not_found"));
    let expected = OwnedTerm::tuple(vec![OwnedTerm::error(), OwnedTerm::atom("not_found")]);

    assert_eq!(result, expected);

    let encoded = encode(&result).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(result, decoded);
}

#[test]
fn test_improper_list_constructor() {
    let improper = OwnedTerm::improper_list(
        vec![OwnedTerm::integer(1), OwnedTerm::integer(2)],
        OwnedTerm::atom("tail"),
    );

    let encoded = encode(&improper).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(improper, decoded);
}

#[test]
fn test_encode_to_writer() {
    let term = OwnedTerm::tuple(vec![OwnedTerm::atom("test"), OwnedTerm::integer(123)]);

    let mut buf = Vec::new();
    erltf::encode_to_writer(&term, &mut buf).unwrap();

    assert!(!buf.is_empty());

    let decoded = decode(&buf).unwrap();
    assert_eq!(term, decoded);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_reference_size_validation() {
    let max_ids: Vec<u32> = vec![0; 65535];
    let valid_ref = OwnedTerm::Reference(ExternalReference::new(
        Atom::new("node@host"),
        1,
        max_ids.clone(),
    ));

    let result = encode(&valid_ref);
    assert!(result.is_ok());

    let too_many_ids: Vec<u32> = vec![0; 65536];
    let invalid_ref = OwnedTerm::Reference(ExternalReference::new(
        Atom::new("node@host"),
        1,
        too_many_ids,
    ));

    let result = encode(&invalid_ref);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("too many IDs"));
    }
}

#[test]
fn test_map_deterministic_encoding() {
    let mut map1 = BTreeMap::new();
    map1.insert(OwnedTerm::atom("z"), OwnedTerm::integer(1));
    map1.insert(OwnedTerm::atom("a"), OwnedTerm::integer(2));
    map1.insert(OwnedTerm::atom("m"), OwnedTerm::integer(3));

    let mut map2 = BTreeMap::new();
    map2.insert(OwnedTerm::atom("a"), OwnedTerm::integer(2));
    map2.insert(OwnedTerm::atom("m"), OwnedTerm::integer(3));
    map2.insert(OwnedTerm::atom("z"), OwnedTerm::integer(1));

    let encoded1 = encode(&OwnedTerm::Map(map1)).unwrap();
    let encoded2 = encode(&OwnedTerm::Map(map2)).unwrap();

    assert_eq!(encoded1, encoded2);
}
