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

use erltf::term::OwnedTerm;
use erltf::types::Atom;
use erltf_serde::{ElixirStruct, from_bytes, from_term, to_bytes, to_term};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, ElixirStruct)]
#[elixir_module = "MyApp.User"]
struct User {
    name: String,
    age: i32,
    active: bool,
}

#[test]
fn test_elixir_struct_serialization() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        active: true,
    };

    let term = to_term(&user).unwrap();

    assert!(term.is_map());
    let map = term.as_map().unwrap();

    let struct_key = OwnedTerm::Atom(Atom::new("__struct__"));
    assert!(map.contains_key(&struct_key));
    assert_eq!(
        map.get(&struct_key),
        Some(&OwnedTerm::Atom(Atom::new("Elixir.MyApp.User")))
    );

    let name_key = OwnedTerm::Atom(Atom::new("name"));
    assert!(map.contains_key(&name_key));
    assert_eq!(
        map.get(&name_key),
        Some(&OwnedTerm::Binary(b"Alice".to_vec()))
    );

    let age_key = OwnedTerm::Atom(Atom::new("age"));
    assert!(map.contains_key(&age_key));
    assert_eq!(map.get(&age_key), Some(&OwnedTerm::Integer(30)));

    let active_key = OwnedTerm::Atom(Atom::new("active"));
    assert!(map.contains_key(&active_key));
    assert_eq!(
        map.get(&active_key),
        Some(&OwnedTerm::Atom(Atom::new("true")))
    );
}

#[test]
fn test_elixir_struct_deserialization() {
    let mut map = BTreeMap::new();
    map.insert(
        OwnedTerm::Atom(Atom::new("__struct__")),
        OwnedTerm::Atom(Atom::new("Elixir.MyApp.User")),
    );
    map.insert(
        OwnedTerm::Atom(Atom::new("name")),
        OwnedTerm::Binary(b"Bob".to_vec()),
    );
    map.insert(OwnedTerm::Atom(Atom::new("age")), OwnedTerm::Integer(25));
    map.insert(
        OwnedTerm::Atom(Atom::new("active")),
        OwnedTerm::Atom(Atom::new("false")),
    );

    let term = OwnedTerm::Map(map);
    let user: User = from_term(&term).unwrap();

    assert_eq!(user.name, "Bob");
    assert_eq!(user.age, 25);
    assert!(!user.active);
}

#[test]
fn test_elixir_struct_roundtrip() {
    let original = User {
        name: "Charlie".to_string(),
        age: 42,
        active: true,
    };

    let bytes = to_bytes(&original).unwrap();
    let decoded: User = from_bytes(&bytes).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn test_elixir_struct_wrong_module_fails() {
    let mut map = BTreeMap::new();
    map.insert(
        OwnedTerm::Atom(Atom::new("__struct__")),
        OwnedTerm::Atom(Atom::new("Elixir.WrongModule")),
    );
    map.insert(
        OwnedTerm::Atom(Atom::new("name")),
        OwnedTerm::Binary(b"Test".to_vec()),
    );
    map.insert(OwnedTerm::Atom(Atom::new("age")), OwnedTerm::Integer(0));
    map.insert(
        OwnedTerm::Atom(Atom::new("active")),
        OwnedTerm::Atom(Atom::new("true")),
    );

    let term = OwnedTerm::Map(map);
    let result: Result<User, _> = from_term(&term);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Elixir.MyApp.User"));
    assert!(err.contains("Elixir.WrongModule"));
}

#[derive(Debug, PartialEq, ElixirStruct)]
#[elixir_module = "MyApp.Nested.DeepModule"]
struct DeepNested {
    value: i64,
}

#[test]
fn test_elixir_struct_nested_module_name() {
    let item = DeepNested { value: 123 };

    let term = to_term(&item).unwrap();
    let map = term.as_map().unwrap();

    let struct_key = OwnedTerm::Atom(Atom::new("__struct__"));
    assert_eq!(
        map.get(&struct_key),
        Some(&OwnedTerm::Atom(Atom::new(
            "Elixir.MyApp.Nested.DeepModule"
        )))
    );
}

#[derive(Debug, PartialEq, ElixirStruct)]
#[elixir_module = "MyApp.WithOptional"]
struct WithOptional {
    required: String,
    optional: Option<i32>,
}

#[test]
fn test_elixir_struct_with_option_some() {
    let item = WithOptional {
        required: "test".to_string(),
        optional: Some(42),
    };

    let bytes = to_bytes(&item).unwrap();
    let decoded: WithOptional = from_bytes(&bytes).unwrap();

    assert_eq!(item, decoded);
}

#[test]
fn test_elixir_struct_with_option_none() {
    let item = WithOptional {
        required: "test".to_string(),
        optional: None,
    };

    let bytes = to_bytes(&item).unwrap();
    let decoded: WithOptional = from_bytes(&bytes).unwrap();

    assert_eq!(item, decoded);
}

#[derive(Debug, PartialEq, ElixirStruct)]
#[elixir_module = "MyApp.WithVec"]
struct WithVec {
    items: Vec<i32>,
}

#[test]
fn test_elixir_struct_with_vec() {
    let item = WithVec {
        items: vec![1, 2, 3, 4, 5],
    };

    let bytes = to_bytes(&item).unwrap();
    let decoded: WithVec = from_bytes(&bytes).unwrap();

    assert_eq!(item, decoded);
}

#[derive(Debug, PartialEq, ElixirStruct)]
#[elixir_module = "MyApp.Inner"]
struct Inner {
    value: String,
}

#[derive(Debug, PartialEq, ElixirStruct)]
#[elixir_module = "MyApp.Outer"]
struct Outer {
    inner: Inner,
    count: i32,
}

#[test]
fn test_nested_elixir_structs() {
    let item = Outer {
        inner: Inner {
            value: "nested".to_string(),
        },
        count: 10,
    };

    let bytes = to_bytes(&item).unwrap();
    let decoded: Outer = from_bytes(&bytes).unwrap();

    assert_eq!(item, decoded);
}

#[test]
fn test_nested_elixir_struct_term_structure() {
    let item = Outer {
        inner: Inner {
            value: "nested".to_string(),
        },
        count: 10,
    };

    let term = to_term(&item).unwrap();
    let map = term.as_map().unwrap();

    let struct_key = OwnedTerm::Atom(Atom::new("__struct__"));
    assert_eq!(
        map.get(&struct_key),
        Some(&OwnedTerm::Atom(Atom::new("Elixir.MyApp.Outer")))
    );

    let inner_key = OwnedTerm::Atom(Atom::new("inner"));
    let inner_term = map.get(&inner_key).unwrap();
    let inner_map = inner_term.as_map().unwrap();

    assert_eq!(
        inner_map.get(&struct_key),
        Some(&OwnedTerm::Atom(Atom::new("Elixir.MyApp.Inner")))
    );
}
