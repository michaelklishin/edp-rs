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
use erltf_serde::from_proplist;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq)]
struct Person {
    name: String,
    age: i64,
}

fn make_person_proplist() -> OwnedTerm {
    OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("name")),
            OwnedTerm::String("Alice".to_string()),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("age")),
            OwnedTerm::Integer(30),
        ]),
    ])
}

#[test]
fn test_from_proplist_to_struct() {
    let proplist = make_person_proplist();
    let person: Person = from_proplist(&proplist).unwrap();

    assert_eq!(person.name, "Alice");
    assert_eq!(person.age, 30);
}

#[test]
fn test_from_proplist_with_binary_keys() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Binary(b"name".to_vec()),
            OwnedTerm::String("Bob".to_string()),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Binary(b"age".to_vec()),
            OwnedTerm::Integer(25),
        ]),
    ]);

    let person: Person = from_proplist(&proplist).unwrap();
    assert_eq!(person.name, "Bob");
    assert_eq!(person.age, 25);
}

#[test]
fn test_from_proplist_to_hashmap() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::String("key1".to_string()),
            OwnedTerm::Integer(1),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::String("key2".to_string()),
            OwnedTerm::Integer(2),
        ]),
    ]);

    let map: HashMap<String, i64> = from_proplist(&proplist).unwrap();
    assert_eq!(map.get("key1"), Some(&1));
    assert_eq!(map.get("key2"), Some(&2));
}

#[test]
fn test_from_proplist_empty() {
    let proplist = OwnedTerm::List(vec![]);
    let map: HashMap<String, i64> = from_proplist(&proplist).unwrap();
    assert!(map.is_empty());
}

#[test]
fn test_from_proplist_nil() {
    let map: HashMap<String, i64> = from_proplist(&OwnedTerm::Nil).unwrap();
    assert!(map.is_empty());
}

#[test]
fn test_from_proplist_skips_malformed_elements() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("name")),
            OwnedTerm::String("Charlie".to_string()),
        ]),
        OwnedTerm::Integer(42),
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("single"))]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("age")),
            OwnedTerm::Integer(35),
        ]),
    ]);

    let person: Person = from_proplist(&proplist).unwrap();
    assert_eq!(person.name, "Charlie");
    assert_eq!(person.age, 35);
}

#[test]
fn test_from_proplist_error_on_non_list() {
    let not_list = OwnedTerm::Integer(42);
    let result: Result<Person, _> = from_proplist(&not_list);
    assert!(result.is_err());
}

#[test]
fn test_from_proplist_with_nested_values() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Outer {
        inner: Vec<i64>,
    }

    let proplist = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("inner")),
        OwnedTerm::List(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::Integer(2),
            OwnedTerm::Integer(3),
        ]),
    ])]);

    let outer: Outer = from_proplist(&proplist).unwrap();
    assert_eq!(outer.inner, vec![1, 2, 3]);
}

#[test]
fn test_from_proplist_with_optional_fields() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct OptionalFields {
        required: String,
        #[serde(default)]
        optional: Option<i64>,
    }

    let proplist = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("required")),
        OwnedTerm::String("present".to_string()),
    ])]);

    let result: OptionalFields = from_proplist(&proplist).unwrap();
    assert_eq!(result.required, "present");
    assert_eq!(result.optional, None);
}

#[test]
fn test_from_proplist_with_rename() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Renamed {
        #[serde(rename = "user_name")]
        name: String,
    }

    let proplist = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("user_name")),
        OwnedTerm::String("renamed".to_string()),
    ])]);

    let result: Renamed = from_proplist(&proplist).unwrap();
    assert_eq!(result.name, "renamed");
}

#[test]
fn test_from_proplist_with_nested_struct() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Address {
        city: String,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct PersonWithAddress {
        name: String,
        address: Address,
    }

    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("name")),
            OwnedTerm::String("Dana".to_string()),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("address")),
            OwnedTerm::Map(
                [(
                    OwnedTerm::Atom(Atom::new("city")),
                    OwnedTerm::String("Paris".to_string()),
                )]
                .into_iter()
                .collect(),
            ),
        ]),
    ]);

    let result: PersonWithAddress = from_proplist(&proplist).unwrap();
    assert_eq!(result.name, "Dana");
    assert_eq!(result.address.city, "Paris");
}

#[test]
fn test_from_proplist_with_bare_atoms() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Atom(Atom::new("debug")),
        OwnedTerm::Atom(Atom::new("verbose")),
    ]);

    let flags: HashMap<String, bool> = from_proplist(&proplist).unwrap();
    assert_eq!(flags.get("debug"), Some(&true));
    assert_eq!(flags.get("verbose"), Some(&true));
}

#[test]
fn test_from_proplist_duplicate_keys_last_wins() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("key")),
            OwnedTerm::Integer(1),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("key")),
            OwnedTerm::Integer(2),
        ]),
    ]);

    let map: HashMap<String, i64> = from_proplist(&proplist).unwrap();
    assert_eq!(map.get("key"), Some(&2));
}

#[test]
fn test_from_proplist_mixed_bare_atoms_and_tuples() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct MixedConfig {
        enabled: bool,
        count: i64,
    }

    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Atom(Atom::new("enabled")),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("count")),
            OwnedTerm::Integer(5),
        ]),
    ]);

    let config: MixedConfig = from_proplist(&proplist).unwrap();
    assert!(config.enabled);
    assert_eq!(config.count, 5);
}
