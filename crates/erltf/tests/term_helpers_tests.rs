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
use std::collections::BTreeMap;

#[test]
fn test_proplist_get_finds_value() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("name")),
            OwnedTerm::String("Alice".to_string()),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("age")),
            OwnedTerm::Integer(30),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("city")),
            OwnedTerm::String("Paris".to_string()),
        ]),
    ]);

    assert_eq!(
        proplist.proplist_get_atom_key("name"),
        Some(&OwnedTerm::String("Alice".to_string()))
    );
    assert_eq!(
        proplist.proplist_get_atom_key("age"),
        Some(&OwnedTerm::Integer(30))
    );
    assert_eq!(
        proplist.proplist_get_atom_key("city"),
        Some(&OwnedTerm::String("Paris".to_string()))
    );
}

#[test]
fn test_proplist_get_not_found() {
    let proplist = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("name")),
        OwnedTerm::String("Bob".to_string()),
    ])]);

    assert_eq!(proplist.proplist_get_atom_key("nonexistent"), None);
}

#[test]
fn test_proplist_get_empty_list() {
    let proplist = OwnedTerm::List(vec![]);
    assert_eq!(proplist.proplist_get_atom_key("anything"), None);
}

#[test]
fn test_proplist_get_on_non_list() {
    let not_a_list = OwnedTerm::Integer(42);
    assert_eq!(not_a_list.proplist_get_atom_key("key"), None);
}

#[test]
fn test_proplist_get_malformed_tuples() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("valid")),
            OwnedTerm::Integer(1),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("too_many")),
            OwnedTerm::Integer(2),
            OwnedTerm::Integer(3),
        ]),
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("lonely"))]),
        OwnedTerm::Integer(42),
    ]);

    assert_eq!(
        proplist.proplist_get_atom_key("valid"),
        Some(&OwnedTerm::Integer(1))
    );
    assert_eq!(proplist.proplist_get_atom_key("too_many"), None);
    assert_eq!(proplist.proplist_get_atom_key("lonely"), None);
}

#[test]
fn test_map_get_atom_finds_value() {
    let mut map = BTreeMap::new();
    map.insert(
        OwnedTerm::Atom(Atom::new("name")),
        OwnedTerm::String("Charlie".to_string()),
    );
    map.insert(OwnedTerm::Atom(Atom::new("age")), OwnedTerm::Integer(25));
    map.insert(
        OwnedTerm::Atom(Atom::new("city")),
        OwnedTerm::String("London".to_string()),
    );
    let map_term = OwnedTerm::Map(map);

    assert_eq!(
        map_term.map_get_atom_key("name"),
        Some(&OwnedTerm::String("Charlie".to_string()))
    );
    assert_eq!(
        map_term.map_get_atom_key("age"),
        Some(&OwnedTerm::Integer(25))
    );
    assert_eq!(
        map_term.map_get_atom_key("city"),
        Some(&OwnedTerm::String("London".to_string()))
    );
}

#[test]
fn test_map_get_atom_not_found() {
    let mut map = BTreeMap::new();
    map.insert(OwnedTerm::Atom(Atom::new("key")), OwnedTerm::Integer(42));
    let map_term = OwnedTerm::Map(map);

    assert_eq!(map_term.map_get_atom_key("nonexistent"), None);
}

#[test]
fn test_map_get_atom_empty_map() {
    let map_term = OwnedTerm::Map(BTreeMap::new());
    assert_eq!(map_term.map_get_atom_key("anything"), None);
}

#[test]
fn test_map_get_atom_on_non_map() {
    let not_a_map = OwnedTerm::Integer(42);
    assert_eq!(not_a_map.map_get_atom_key("key"), None);
}

#[test]
fn test_as_erlang_string_from_integer_list() {
    let term = OwnedTerm::List(vec![
        OwnedTerm::Integer(72),
        OwnedTerm::Integer(101),
        OwnedTerm::Integer(108),
        OwnedTerm::Integer(108),
        OwnedTerm::Integer(111),
    ]);

    assert_eq!(term.as_erlang_string(), Some("Hello".to_string()));
}

#[test]
fn test_as_erlang_string_from_string() {
    let term = OwnedTerm::String("World".to_string());
    assert_eq!(term.as_erlang_string(), Some("World".to_string()));
}

#[test]
fn test_as_erlang_string_from_binary() {
    let term = OwnedTerm::Binary(vec![82, 117, 115, 116]);
    assert_eq!(term.as_erlang_string(), Some("Rust".to_string()));
}

#[test]
fn test_as_erlang_string_invalid_integer_list() {
    let term = OwnedTerm::List(vec![
        OwnedTerm::Integer(72),
        OwnedTerm::Integer(256),
        OwnedTerm::Integer(108),
    ]);

    assert_eq!(term.as_erlang_string(), None);
}

#[test]
fn test_as_erlang_string_mixed_list() {
    let term = OwnedTerm::List(vec![
        OwnedTerm::Integer(72),
        OwnedTerm::Atom(Atom::new("not_an_int")),
        OwnedTerm::Integer(108),
    ]);

    assert_eq!(term.as_erlang_string(), None);
}

#[test]
fn test_as_erlang_string_on_non_string_types() {
    assert_eq!(OwnedTerm::Integer(42).as_erlang_string(), None);
    assert_eq!(OwnedTerm::Atom(Atom::new("atom")).as_erlang_string(), None);
    assert_eq!(OwnedTerm::Float(2.5).as_erlang_string(), None);
}
