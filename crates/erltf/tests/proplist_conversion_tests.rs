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

fn make_proplist() -> OwnedTerm {
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
fn test_is_proplist_valid() {
    let proplist = make_proplist();
    assert!(proplist.is_proplist());
}

#[test]
fn test_is_proplist_empty() {
    let empty = OwnedTerm::List(vec![]);
    assert!(empty.is_proplist());
    assert!(OwnedTerm::Nil.is_proplist());
}

#[test]
fn test_is_proplist_with_bare_atoms() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Atom(Atom::new("debug")),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("level")),
            OwnedTerm::Integer(5),
        ]),
    ]);
    assert!(proplist.is_proplist());
}

#[test]
fn test_is_proplist_with_binary_keys() {
    let proplist = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
        OwnedTerm::Binary(b"key".to_vec()),
        OwnedTerm::Integer(1),
    ])]);
    assert!(proplist.is_proplist());
}

#[test]
fn test_is_proplist_invalid_not_tuples() {
    let not_proplist = OwnedTerm::List(vec![OwnedTerm::Integer(1), OwnedTerm::Integer(2)]);
    assert!(!not_proplist.is_proplist());
}

#[test]
fn test_is_proplist_invalid_wrong_tuple_size() {
    let not_proplist = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("a")),
        OwnedTerm::Integer(1),
        OwnedTerm::Integer(2),
    ])]);
    assert!(!not_proplist.is_proplist());
}

#[test]
fn test_is_proplist_on_non_list() {
    assert!(!OwnedTerm::Integer(42).is_proplist());
    assert!(!OwnedTerm::Map(BTreeMap::new()).is_proplist());
}

#[test]
fn test_normalize_proplist_expands_bare_atoms() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Atom(Atom::new("debug")),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("level")),
            OwnedTerm::Integer(5),
        ]),
    ]);

    let normalized = proplist.normalize_proplist().unwrap();

    if let OwnedTerm::List(elements) = normalized {
        assert_eq!(elements.len(), 2);
        if let OwnedTerm::Tuple(t) = &elements[0] {
            assert_eq!(t[0], OwnedTerm::Atom(Atom::new("debug")));
            assert_eq!(t[1], OwnedTerm::Atom(Atom::new("true")));
        } else {
            panic!("expected tuple");
        }
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_normalize_proplist_filters_invalid() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("valid")),
            OwnedTerm::Integer(1),
        ]),
        OwnedTerm::Integer(42),
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("single"))]),
    ]);

    let normalized = proplist.normalize_proplist().unwrap();
    if let OwnedTerm::List(elements) = normalized {
        assert_eq!(elements.len(), 1);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_proplist_to_map() {
    let proplist = make_proplist();
    let map = proplist.proplist_to_map().unwrap();

    if let OwnedTerm::Map(m) = map {
        assert_eq!(m.len(), 2);
        assert_eq!(
            m.get(&OwnedTerm::Atom(Atom::new("name"))),
            Some(&OwnedTerm::String("Alice".to_string()))
        );
        assert_eq!(
            m.get(&OwnedTerm::Atom(Atom::new("age"))),
            Some(&OwnedTerm::Integer(30))
        );
    } else {
        panic!("expected map");
    }
}

#[test]
fn test_proplist_to_map_with_bare_atoms() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Atom(Atom::new("debug")),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("level")),
            OwnedTerm::Integer(5),
        ]),
    ]);

    let map = proplist.proplist_to_map().unwrap();
    if let OwnedTerm::Map(m) = map {
        assert_eq!(
            m.get(&OwnedTerm::Atom(Atom::new("debug"))),
            Some(&OwnedTerm::Atom(Atom::new("true")))
        );
    } else {
        panic!("expected map");
    }
}

#[test]
fn test_proplist_to_map_already_map() {
    let mut m = BTreeMap::new();
    m.insert(OwnedTerm::Atom(Atom::new("key")), OwnedTerm::Integer(1));
    let map = OwnedTerm::Map(m);
    let result = map.proplist_to_map().unwrap();
    assert!(matches!(result, OwnedTerm::Map(_)));
}

#[test]
fn test_map_to_proplist() {
    let mut m = BTreeMap::new();
    m.insert(
        OwnedTerm::Atom(Atom::new("name")),
        OwnedTerm::String("Bob".to_string()),
    );
    m.insert(OwnedTerm::Atom(Atom::new("age")), OwnedTerm::Integer(25));
    let map = OwnedTerm::Map(m);

    let proplist = map.map_to_proplist().unwrap();
    if let OwnedTerm::List(elements) = proplist {
        assert_eq!(elements.len(), 2);
        for el in &elements {
            assert!(matches!(el, OwnedTerm::Tuple(t) if t.len() == 2));
        }
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_map_to_proplist_already_list() {
    let proplist = make_proplist();
    let result = proplist.map_to_proplist().unwrap();
    assert!(matches!(result, OwnedTerm::List(_)));
}

#[test]
fn test_to_map_recursive_nested_proplists() {
    let nested = OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("outer")),
        OwnedTerm::List(vec![OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("inner")),
            OwnedTerm::Integer(42),
        ])]),
    ])]);

    let result = nested.to_map_recursive().unwrap();
    if let OwnedTerm::Map(outer) = result {
        if let Some(OwnedTerm::Map(inner)) = outer.get(&OwnedTerm::Atom(Atom::new("outer"))) {
            assert_eq!(
                inner.get(&OwnedTerm::Atom(Atom::new("inner"))),
                Some(&OwnedTerm::Integer(42))
            );
        } else {
            panic!("expected inner map");
        }
    } else {
        panic!("expected outer map");
    }
}

#[test]
fn test_to_map_recursive_preserves_non_proplists() {
    let list_of_integers = OwnedTerm::List(vec![OwnedTerm::Integer(1), OwnedTerm::Integer(2)]);

    let result = list_of_integers.to_map_recursive().unwrap();
    assert!(matches!(result, OwnedTerm::List(_)));
}

#[test]
fn test_atomize_keys_proplist() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Binary(b"name".to_vec()),
            OwnedTerm::String("Alice".to_string()),
        ]),
        OwnedTerm::Tuple(vec![
            OwnedTerm::String("age".to_string()),
            OwnedTerm::Integer(30),
        ]),
    ]);

    let result = proplist.atomize_keys().unwrap();
    if let OwnedTerm::List(elements) = result {
        for el in &elements {
            if let OwnedTerm::Tuple(t) = el {
                assert!(matches!(&t[0], OwnedTerm::Atom(_)));
            }
        }
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_atomize_keys_map() {
    let mut m = BTreeMap::new();
    m.insert(
        OwnedTerm::Binary(b"name".to_vec()),
        OwnedTerm::String("Bob".to_string()),
    );
    let map = OwnedTerm::Map(m);

    let result = map.atomize_keys().unwrap();
    if let OwnedTerm::Map(m) = result {
        assert!(m.contains_key(&OwnedTerm::Atom(Atom::new("name"))));
    } else {
        panic!("expected map");
    }
}

#[test]
fn test_atomize_keys_drops_non_convertible() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Tuple(vec![
            OwnedTerm::Binary(b"valid".to_vec()),
            OwnedTerm::Integer(1),
        ]),
        OwnedTerm::Tuple(vec![OwnedTerm::Integer(42), OwnedTerm::Integer(2)]),
        OwnedTerm::Tuple(vec![OwnedTerm::List(vec![]), OwnedTerm::Integer(3)]),
    ]);

    let result = proplist.atomize_keys().unwrap();
    if let OwnedTerm::List(elements) = result {
        assert_eq!(elements.len(), 1);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_proplist_to_map_duplicate_keys_last_wins() {
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

    let map = proplist.proplist_to_map().unwrap();
    if let OwnedTerm::Map(m) = map {
        assert_eq!(
            m.get(&OwnedTerm::Atom(Atom::new("key"))),
            Some(&OwnedTerm::Integer(2))
        );
    } else {
        panic!("expected map");
    }
}

#[test]
fn test_as_list_wrapped_already_list() {
    let list = OwnedTerm::List(vec![OwnedTerm::Integer(1)]);
    let result = list.as_list_wrapped();
    if let OwnedTerm::List(elements) = result {
        assert_eq!(elements.len(), 1);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_as_list_wrapped_wraps_non_list() {
    let term = OwnedTerm::Integer(42);
    let result = term.as_list_wrapped();
    if let OwnedTerm::List(elements) = result {
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0], OwnedTerm::Integer(42));
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_as_list_wrapped_nil() {
    let result = OwnedTerm::Nil.as_list_wrapped();
    assert!(matches!(result, OwnedTerm::Nil));
}

#[test]
fn test_proplist_iter() {
    let proplist = make_proplist();
    let pairs: Vec<_> = proplist.proplist_iter().unwrap().collect();

    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0].0, &OwnedTerm::Atom(Atom::new("name")));
    assert_eq!(pairs[0].1, &OwnedTerm::String("Alice".to_string()));
}

#[test]
fn test_proplist_iter_with_bare_atoms() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Atom(Atom::new("debug")),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("level")),
            OwnedTerm::Integer(5),
        ]),
    ]);

    let pairs: Vec<_> = proplist.proplist_iter().unwrap().collect();
    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0].0, &OwnedTerm::Atom(Atom::new("debug")));
    assert_eq!(pairs[0].1, &OwnedTerm::Atom(Atom::new("true")));
}

#[test]
fn test_proplist_iter_skips_invalid() {
    let proplist = OwnedTerm::List(vec![
        OwnedTerm::Integer(42),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("valid")),
            OwnedTerm::Integer(1),
        ]),
    ]);

    let pairs: Vec<_> = proplist.proplist_iter().unwrap().collect();
    assert_eq!(pairs.len(), 1);
}

#[test]
fn test_proplist_iter_on_non_list() {
    assert!(OwnedTerm::Integer(42).proplist_iter().is_none());
}
