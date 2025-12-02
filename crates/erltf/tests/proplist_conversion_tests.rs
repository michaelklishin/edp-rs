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
use erltf::{erl_atom, erl_int, erl_list, erl_map, erl_tuple};
use std::collections::BTreeMap;

fn make_proplist() -> OwnedTerm {
    erl_list![
        erl_tuple![erl_atom!("name"), OwnedTerm::String("Alice".to_string())],
        erl_tuple![erl_atom!("age"), erl_int!(30)]
    ]
}

#[test]
fn test_is_proplist_valid() {
    assert!(make_proplist().is_proplist());
}

#[test]
fn test_is_proplist_empty() {
    assert!(erl_list![].is_proplist());
    assert!(OwnedTerm::Nil.is_proplist());
}

#[test]
fn test_is_proplist_with_bare_atoms() {
    let proplist = erl_list![
        erl_atom!("debug"),
        erl_tuple![erl_atom!("level"), erl_int!(5)]
    ];
    assert!(proplist.is_proplist());
}

#[test]
fn test_is_proplist_with_binary_keys() {
    let proplist = erl_list![erl_tuple![OwnedTerm::Binary(b"key".to_vec()), erl_int!(1)]];
    assert!(proplist.is_proplist());
}

#[test]
fn test_is_proplist_invalid_not_tuples() {
    let not_proplist = erl_list![erl_int!(1), erl_int!(2)];
    assert!(!not_proplist.is_proplist());
}

#[test]
fn test_is_proplist_invalid_wrong_tuple_size() {
    let not_proplist = erl_list![erl_tuple![erl_atom!("a"), erl_int!(1), erl_int!(2)]];
    assert!(!not_proplist.is_proplist());
}

#[test]
fn test_is_proplist_on_non_list() {
    assert!(!erl_int!(42).is_proplist());
    assert!(!erl_map! {}.is_proplist());
}

#[test]
fn test_normalize_proplist_expands_bare_atoms() {
    let proplist = erl_list![
        erl_atom!("debug"),
        erl_tuple![erl_atom!("level"), erl_int!(5)]
    ];

    let normalized = proplist.normalize_proplist().unwrap();

    if let OwnedTerm::List(elements) = normalized {
        assert_eq!(elements.len(), 2);
        if let OwnedTerm::Tuple(t) = &elements[0] {
            assert_eq!(t[0], erl_atom!("debug"));
            assert_eq!(t[1], erl_atom!("true"));
        } else {
            panic!("expected tuple");
        }
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_normalize_proplist_filters_invalid() {
    let proplist = erl_list![
        erl_tuple![erl_atom!("valid"), erl_int!(1)],
        erl_int!(42),
        erl_tuple![erl_atom!("single")]
    ];

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
            m.get(&erl_atom!("name")),
            Some(&OwnedTerm::String("Alice".to_string()))
        );
        assert_eq!(m.get(&erl_atom!("age")), Some(&erl_int!(30)));
    } else {
        panic!("expected map");
    }
}

#[test]
fn test_proplist_to_map_with_bare_atoms() {
    let proplist = erl_list![
        erl_atom!("debug"),
        erl_tuple![erl_atom!("level"), erl_int!(5)]
    ];

    let map = proplist.proplist_to_map().unwrap();
    if let OwnedTerm::Map(m) = map {
        assert_eq!(m.get(&erl_atom!("debug")), Some(&erl_atom!("true")));
    } else {
        panic!("expected map");
    }
}

#[test]
fn test_proplist_to_map_already_map() {
    let map = erl_map! { erl_atom!("key") => erl_int!(1) };
    let result = map.proplist_to_map().unwrap();
    assert!(matches!(result, OwnedTerm::Map(_)));
}

#[test]
fn test_map_to_proplist() {
    let map = erl_map! {
        erl_atom!("name") => OwnedTerm::String("Bob".to_string()),
        erl_atom!("age") => erl_int!(25)
    };

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
    let result = make_proplist().map_to_proplist().unwrap();
    assert!(matches!(result, OwnedTerm::List(_)));
}

#[test]
fn test_to_map_recursive_nested_proplists() {
    let nested = erl_list![erl_tuple![
        erl_atom!("outer"),
        erl_list![erl_tuple![erl_atom!("inner"), erl_int!(42)]]
    ]];

    let result = nested.to_map_recursive().unwrap();
    if let OwnedTerm::Map(outer) = result {
        if let Some(OwnedTerm::Map(inner)) = outer.get(&erl_atom!("outer")) {
            assert_eq!(inner.get(&erl_atom!("inner")), Some(&erl_int!(42)));
        } else {
            panic!("expected inner map");
        }
    } else {
        panic!("expected outer map");
    }
}

#[test]
fn test_to_map_recursive_preserves_non_proplists() {
    let list_of_integers = erl_list![erl_int!(1), erl_int!(2)];
    let result = list_of_integers.to_map_recursive().unwrap();
    assert!(matches!(result, OwnedTerm::List(_)));
}

#[test]
fn test_atomize_keys_proplist() {
    let proplist = erl_list![
        erl_tuple![
            OwnedTerm::Binary(b"name".to_vec()),
            OwnedTerm::String("Alice".to_string())
        ],
        erl_tuple![OwnedTerm::String("age".to_string()), erl_int!(30)]
    ];

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
        assert!(m.contains_key(&erl_atom!("name")));
    } else {
        panic!("expected map");
    }
}

#[test]
fn test_atomize_keys_drops_non_convertible() {
    let proplist = erl_list![
        erl_tuple![OwnedTerm::Binary(b"valid".to_vec()), erl_int!(1)],
        erl_tuple![erl_int!(42), erl_int!(2)],
        erl_tuple![erl_list![], erl_int!(3)]
    ];

    let result = proplist.atomize_keys().unwrap();
    if let OwnedTerm::List(elements) = result {
        assert_eq!(elements.len(), 1);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_proplist_to_map_duplicate_keys_last_wins() {
    let proplist = erl_list![
        erl_tuple![erl_atom!("key"), erl_int!(1)],
        erl_tuple![erl_atom!("key"), erl_int!(2)]
    ];

    let map = proplist.proplist_to_map().unwrap();
    if let OwnedTerm::Map(m) = map {
        assert_eq!(m.get(&erl_atom!("key")), Some(&erl_int!(2)));
    } else {
        panic!("expected map");
    }
}

#[test]
fn test_as_list_wrapped_already_list() {
    let list = erl_list![erl_int!(1)];
    let result = list.as_list_wrapped();
    if let OwnedTerm::List(elements) = result {
        assert_eq!(elements.len(), 1);
    } else {
        panic!("expected list");
    }
}

#[test]
fn test_as_list_wrapped_wraps_non_list() {
    let term = erl_int!(42);
    let result = term.as_list_wrapped();
    if let OwnedTerm::List(elements) = result {
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0], erl_int!(42));
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
    assert_eq!(pairs[0].0, &erl_atom!("name"));
    assert_eq!(pairs[0].1, &OwnedTerm::String("Alice".to_string()));
}

#[test]
fn test_proplist_iter_with_bare_atoms() {
    let proplist = erl_list![
        erl_atom!("debug"),
        erl_tuple![erl_atom!("level"), erl_int!(5)]
    ];

    let pairs: Vec<_> = proplist.proplist_iter().unwrap().collect();
    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0].0, &erl_atom!("debug"));
    assert_eq!(pairs[0].1, &erl_atom!("true"));
}

#[test]
fn test_proplist_iter_skips_invalid() {
    let proplist = erl_list![erl_int!(42), erl_tuple![erl_atom!("valid"), erl_int!(1)]];

    let pairs: Vec<_> = proplist.proplist_iter().unwrap().collect();
    assert_eq!(pairs.len(), 1);
}

#[test]
fn test_proplist_iter_on_non_list() {
    assert!(erl_int!(42).proplist_iter().is_none());
}
