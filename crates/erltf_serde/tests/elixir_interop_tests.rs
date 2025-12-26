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

#![cfg(feature = "elixir-interop")]

use erltf::term::OwnedTerm;
use erltf::types::Atom;
use erltf_serde::{from_term, to_term};
use serde::{Deserialize, Serialize};

#[test]
fn test_serialize_none_emits_nil() {
    let val: Option<i32> = None;
    let term = to_term(&val).unwrap();

    match term {
        OwnedTerm::Atom(atom) => assert_eq!(atom.as_str(), "nil"),
        _ => panic!("Expected atom, got {:?}", term),
    }
}

#[test]
fn test_deserialize_nil_as_none() {
    let term = OwnedTerm::Atom(Atom::new("nil"));
    let result: Option<i32> = from_term(&term).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_deserialize_undefined_as_none() {
    let term = OwnedTerm::Atom(Atom::new("undefined"));
    let result: Option<i32> = from_term(&term).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_roundtrip_none() {
    let val: Option<i32> = None;
    let term = to_term(&val).unwrap();
    let result: Option<i32> = from_term(&term).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_roundtrip_some() {
    let val: Option<i32> = Some(42);
    let term = to_term(&val).unwrap();
    let result: Option<i32> = from_term(&term).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_option_string_none() {
    let val: Option<String> = None;
    let term = to_term(&val).unwrap();
    assert!(matches!(term, OwnedTerm::Atom(ref a) if a.as_str() == "nil"));
    let result: Option<String> = from_term(&term).unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_option_string_some() {
    let val: Option<String> = Some("hello".to_string());
    let term = to_term(&val).unwrap();
    let result: Option<String> = from_term(&term).unwrap();
    assert_eq!(result, Some("hello".to_string()));
}

#[test]
fn test_vec_of_options() {
    let val: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    let term = to_term(&val).unwrap();
    let result: Vec<Option<i32>> = from_term(&term).unwrap();
    assert_eq!(result, vec![Some(1), None, Some(3)]);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct WithOption {
    name: String,
    value: Option<i32>,
}

#[test]
fn test_struct_with_option_none() {
    let val = WithOption {
        name: "test".to_string(),
        value: None,
    };
    let term = to_term(&val).unwrap();
    let result: WithOption = from_term(&term).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_struct_with_option_some() {
    let val = WithOption {
        name: "test".to_string(),
        value: Some(42),
    };
    let term = to_term(&val).unwrap();
    let result: WithOption = from_term(&term).unwrap();
    assert_eq!(val, result);
}
