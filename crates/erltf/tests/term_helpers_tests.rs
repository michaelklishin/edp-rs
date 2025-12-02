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
use erltf::types::{Atom, ExternalPid, Mfa};
use erltf::{KeyValueAccess, erl_atom, erl_atoms, erl_int, erl_list, erl_map, erl_tuple};

#[test]
fn test_proplist_get_finds_value() {
    let proplist = erl_list![
        erl_tuple![erl_atom!("name"), OwnedTerm::String("Alice".to_string())],
        erl_tuple![erl_atom!("age"), erl_int!(30)],
        erl_tuple![erl_atom!("city"), OwnedTerm::String("Paris".to_string())]
    ];

    assert_eq!(
        proplist.proplist_get_atom_key("name"),
        Some(&OwnedTerm::String("Alice".to_string()))
    );
    assert_eq!(proplist.proplist_get_atom_key("age"), Some(&erl_int!(30)));
    assert_eq!(
        proplist.proplist_get_atom_key("city"),
        Some(&OwnedTerm::String("Paris".to_string()))
    );
}

#[test]
fn test_proplist_get_not_found() {
    let proplist = erl_list![erl_tuple![
        erl_atom!("name"),
        OwnedTerm::String("Bob".to_string())
    ]];

    assert_eq!(proplist.proplist_get_atom_key("nonexistent"), None);
}

#[test]
fn test_proplist_get_empty_list() {
    let proplist = erl_list![];
    assert_eq!(proplist.proplist_get_atom_key("anything"), None);
}

#[test]
fn test_proplist_get_on_non_list() {
    assert_eq!(erl_int!(42).proplist_get_atom_key("key"), None);
}

#[test]
fn test_proplist_get_malformed_tuples() {
    let proplist = erl_list![
        erl_tuple![erl_atom!("valid"), erl_int!(1)],
        erl_tuple![erl_atom!("too_many"), erl_int!(2), erl_int!(3)],
        erl_tuple![erl_atom!("lonely")],
        erl_int!(42)
    ];

    assert_eq!(proplist.proplist_get_atom_key("valid"), Some(&erl_int!(1)));
    assert_eq!(proplist.proplist_get_atom_key("too_many"), None);
    assert_eq!(proplist.proplist_get_atom_key("lonely"), None);
}

#[test]
fn test_map_get_atom_finds_value() {
    let map_term = erl_map! {
        erl_atom!("name") => OwnedTerm::String("Charlie".to_string()),
        erl_atom!("age") => erl_int!(25),
        erl_atom!("city") => OwnedTerm::String("London".to_string())
    };

    assert_eq!(
        map_term.map_get_atom_key("name"),
        Some(&OwnedTerm::String("Charlie".to_string()))
    );
    assert_eq!(map_term.map_get_atom_key("age"), Some(&erl_int!(25)));
    assert_eq!(
        map_term.map_get_atom_key("city"),
        Some(&OwnedTerm::String("London".to_string()))
    );
}

#[test]
fn test_map_get_atom_not_found() {
    let map_term = erl_map! { erl_atom!("key") => erl_int!(42) };
    assert_eq!(map_term.map_get_atom_key("nonexistent"), None);
}

#[test]
fn test_map_get_atom_empty_map() {
    let map_term = erl_map! {};
    assert_eq!(map_term.map_get_atom_key("anything"), None);
}

#[test]
fn test_map_get_atom_on_non_map() {
    assert_eq!(erl_int!(42).map_get_atom_key("key"), None);
}

#[test]
fn test_as_erlang_string_from_integer_list() {
    let term = erl_list![
        erl_int!(72),
        erl_int!(101),
        erl_int!(108),
        erl_int!(108),
        erl_int!(111)
    ];
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
    let term = erl_list![erl_int!(72), erl_int!(256), erl_int!(108)];
    assert_eq!(term.as_erlang_string(), None);
}

#[test]
fn test_as_erlang_string_mixed_list() {
    let term = erl_list![erl_int!(72), erl_atom!("not_an_int"), erl_int!(108)];
    assert_eq!(term.as_erlang_string(), None);
}

#[test]
fn test_as_erlang_string_on_non_string_types() {
    assert_eq!(erl_int!(42).as_erlang_string(), None);
    assert_eq!(erl_atom!("atom").as_erlang_string(), None);
    assert_eq!(OwnedTerm::Float(2.5).as_erlang_string(), None);
}

#[test]
fn test_charlist_from_ascii() {
    let term = OwnedTerm::charlist("Hello");
    assert_eq!(
        term,
        erl_list![
            erl_int!(72),
            erl_int!(101),
            erl_int!(108),
            erl_int!(108),
            erl_int!(111)
        ]
    );
}

#[test]
fn test_charlist_from_unicode() {
    let term = OwnedTerm::charlist("日本");
    assert_eq!(term, erl_list![erl_int!(0x65E5), erl_int!(0x672C)]);
}

#[test]
fn test_charlist_empty() {
    let term = OwnedTerm::charlist("");
    assert_eq!(term, erl_list![]);
}

#[test]
fn test_is_charlist_valid() {
    let term = erl_list![erl_int!(72), erl_int!(101), erl_int!(108)];
    assert!(term.is_charlist());
}

#[test]
fn test_is_charlist_empty() {
    assert!(erl_list![].is_charlist());
    assert!(OwnedTerm::Nil.is_charlist());
}

#[test]
fn test_is_charlist_negative_integer() {
    let term = erl_list![erl_int!(-1)];
    assert!(!term.is_charlist());
}

#[test]
fn test_is_charlist_non_integer() {
    let term = erl_list![erl_atom!("a")];
    assert!(!term.is_charlist());
}

#[test]
fn test_is_charlist_on_non_list() {
    assert!(!erl_int!(42).is_charlist());
    assert!(!erl_atom!("atom").is_charlist());
}

#[test]
fn test_as_pid() {
    let pid = ExternalPid::new(Atom::new("node@host"), 1, 2, 3);
    let term = OwnedTerm::Pid(pid.clone());
    assert_eq!(term.as_pid(), Some(&pid));
}

#[test]
fn test_as_pid_on_non_pid() {
    assert_eq!(erl_int!(42).as_pid(), None);
}

#[test]
fn test_try_as_pid() {
    let pid = ExternalPid::new(Atom::new("node@host"), 1, 2, 3);
    let term = OwnedTerm::Pid(pid.clone());
    assert_eq!(term.try_as_pid().unwrap(), &pid);
}

#[test]
fn test_try_as_pid_error() {
    assert!(erl_int!(42).try_as_pid().is_err());
}

#[test]
fn test_is_pid() {
    let pid = ExternalPid::new(Atom::new("node@host"), 1, 2, 3);
    assert!(OwnedTerm::Pid(pid).is_pid());
    assert!(!erl_int!(42).is_pid());
}

#[test]
fn test_proplist_get_i64() {
    let proplist = erl_list![erl_tuple![erl_atom!("count"), erl_int!(42)]];
    assert_eq!(proplist.proplist_get_i64("count"), Some(42));
    assert_eq!(proplist.proplist_get_i64("missing"), None);
}

#[test]
fn test_proplist_get_bool() {
    let proplist = erl_list![
        erl_tuple![erl_atom!("enabled"), erl_atom!("true")],
        erl_tuple![erl_atom!("disabled"), erl_atom!("false")]
    ];
    assert_eq!(proplist.proplist_get_bool("enabled"), Some(true));
    assert_eq!(proplist.proplist_get_bool("disabled"), Some(false));
    assert_eq!(proplist.proplist_get_bool("missing"), None);
}

#[test]
fn test_proplist_get_atom() {
    let proplist = erl_list![erl_tuple![erl_atom!("status"), erl_atom!("running")]];
    assert_eq!(
        proplist.proplist_get_atom("status"),
        Some(&Atom::new("running"))
    );
    assert_eq!(proplist.proplist_get_atom("missing"), None);
}

#[test]
fn test_proplist_get_string() {
    let proplist = erl_list![erl_tuple![
        erl_atom!("name"),
        erl_list![erl_int!(66), erl_int!(111), erl_int!(98)]
    ]];
    assert_eq!(
        proplist.proplist_get_string("name"),
        Some("Bob".to_string())
    );
    assert_eq!(proplist.proplist_get_string("missing"), None);
}

#[test]
fn test_proplist_get_pid() {
    let pid = ExternalPid::new(Atom::new("node@host"), 1, 2, 3);
    let proplist = erl_list![erl_tuple![
        erl_atom!("process"),
        OwnedTerm::Pid(pid.clone())
    ]];
    assert_eq!(proplist.proplist_get_pid("process"), Some(&pid));
    assert_eq!(proplist.proplist_get_pid("missing"), None);
}

#[test]
fn test_pid_to_charlist_term() {
    let pid = ExternalPid::new(Atom::new("node@host"), 123, 456, 7);
    let charlist = pid.to_charlist_term();
    assert!(charlist.is_charlist());
    assert_eq!(charlist.as_erlang_string(), Some("<0.123.456>".to_string()));
}

#[test]
fn test_mfa_new() {
    let mfa = Mfa::new("erlang", "self", 0);
    assert_eq!(mfa.module, Atom::new("erlang"));
    assert_eq!(mfa.function, Atom::new("self"));
    assert_eq!(mfa.arity, 0);
}

#[test]
fn test_mfa_display() {
    let mfa = Mfa::new("lists", "map", 2);
    assert_eq!(format!("{}", mfa), "lists:map/2");
}

#[test]
fn test_mfa_to_term() {
    let mfa = Mfa::new("erlang", "node", 0);
    let term = mfa.to_term();
    assert_eq!(
        term,
        erl_tuple![erl_atom!("erlang"), erl_atom!("node"), erl_int!(0)]
    );
}

#[test]
fn test_mfa_try_from_term() {
    let term = erl_tuple![erl_atom!("lists"), erl_atom!("sort"), erl_int!(1)];
    let mfa = Mfa::try_from_term(&term).unwrap();
    assert_eq!(mfa.module, Atom::new("lists"));
    assert_eq!(mfa.function, Atom::new("sort"));
    assert_eq!(mfa.arity, 1);
}

#[test]
fn test_mfa_try_from_term_rejects_list_arity() {
    let term = erl_tuple![
        erl_atom!("io"),
        erl_atom!("format"),
        erl_list![OwnedTerm::String("~p~n".to_string()), erl_list![]]
    ];
    assert!(Mfa::try_from_term(&term).is_none());
}

#[test]
fn test_mfa_try_from_term_invalid() {
    assert!(Mfa::try_from_term(&erl_int!(42)).is_none());
    assert!(Mfa::try_from_term(&erl_tuple![erl_int!(1)]).is_none());
}

#[test]
fn test_as_charlist_string_ascii() {
    let term = OwnedTerm::charlist("Hello");
    assert_eq!(term.as_charlist_string(), Some("Hello".to_string()));
}

#[test]
fn test_as_charlist_string_unicode() {
    let term = OwnedTerm::charlist("日本語");
    assert_eq!(term.as_charlist_string(), Some("日本語".to_string()));
}

#[test]
fn test_as_charlist_string_emoji() {
    let term = OwnedTerm::charlist("🦀");
    assert_eq!(term.as_charlist_string(), Some("🦀".to_string()));
}

#[test]
fn test_as_charlist_string_empty() {
    assert_eq!(erl_list![].as_charlist_string(), Some(String::new()));
    assert_eq!(OwnedTerm::Nil.as_charlist_string(), Some(String::new()));
}

#[test]
fn test_as_charlist_string_from_string_term() {
    let term = OwnedTerm::String("test".to_string());
    assert_eq!(term.as_charlist_string(), Some("test".to_string()));
}

#[test]
fn test_as_charlist_string_from_binary() {
    let term = OwnedTerm::Binary(b"binary".to_vec());
    assert_eq!(term.as_charlist_string(), Some("binary".to_string()));
}

#[test]
fn test_as_charlist_string_invalid_codepoint() {
    let term = erl_list![erl_int!(0x110000)];
    assert_eq!(term.as_charlist_string(), None);
}

#[test]
fn test_as_charlist_string_negative() {
    let term = erl_list![erl_int!(-1)];
    assert_eq!(term.as_charlist_string(), None);
}

#[test]
fn test_as_list_or_empty_list() {
    let list = erl_list![erl_int!(1), erl_int!(2)];
    assert_eq!(list.as_list_or_empty().len(), 2);
}

#[test]
fn test_as_list_or_empty_empty_list() {
    assert!(erl_list![].as_list_or_empty().is_empty());
}

#[test]
fn test_as_list_or_empty_non_list() {
    assert!(erl_int!(42).as_list_or_empty().is_empty());
    assert!(erl_atom!("test").as_list_or_empty().is_empty());
    assert!(OwnedTerm::Nil.as_list_or_empty().is_empty());
}

#[test]
fn test_try_as_mfa() {
    let term = erl_tuple![erl_atom!("erlang"), erl_atom!("node"), erl_int!(0)];
    let mfa = term.try_as_mfa().unwrap();
    assert_eq!(mfa.module, Atom::new("erlang"));
    assert_eq!(mfa.function, Atom::new("node"));
    assert_eq!(mfa.arity, 0);
}

#[test]
fn test_try_as_mfa_invalid() {
    assert!(erl_int!(42).try_as_mfa().is_none());
}

#[test]
fn test_format_as_mfa() {
    let term = erl_tuple![erl_atom!("lists"), erl_atom!("map"), erl_int!(2)];
    assert_eq!(term.format_as_mfa(), Some("lists:map/2".to_string()));
}

#[test]
fn test_format_as_mfa_invalid() {
    assert_eq!(erl_int!(42).format_as_mfa(), None);
}

#[test]
fn test_format_as_pid() {
    let pid = ExternalPid::new(Atom::new("node@host"), 123, 456, 0);
    let term = OwnedTerm::Pid(pid);
    assert_eq!(term.format_as_pid(), Some("<123.456.0>".to_string()));
}

#[test]
fn test_format_as_pid_non_pid() {
    assert_eq!(erl_int!(42).format_as_pid(), None);
}

#[test]
fn test_proplist_get_atom_string() {
    let proplist = erl_list![erl_tuple![erl_atom!("status"), erl_atom!("running")]];
    assert_eq!(
        proplist.proplist_get_atom_string("status"),
        Some("running".to_string())
    );
    assert_eq!(proplist.proplist_get_atom_string("missing"), None);
}

#[test]
fn test_proplist_get_atom_string_or() {
    let proplist = erl_list![erl_tuple![erl_atom!("status"), erl_atom!("running")]];
    assert_eq!(
        proplist.proplist_get_atom_string_or("status", "unknown"),
        "running".to_string()
    );
    assert_eq!(
        proplist.proplist_get_atom_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_proplist_get_pid_string() {
    let pid = ExternalPid::new(Atom::new("node@host"), 100, 200, 0);
    let proplist = erl_list![erl_tuple![erl_atom!("group_leader"), OwnedTerm::Pid(pid)]];
    assert_eq!(
        proplist.proplist_get_pid_string("group_leader"),
        Some("<100.200.0>".to_string())
    );
    assert_eq!(proplist.proplist_get_pid_string("missing"), None);
}

#[test]
fn test_proplist_get_mfa_string() {
    let proplist = erl_list![erl_tuple![
        erl_atom!("initial_call"),
        erl_tuple![erl_atom!("gen_server"), erl_atom!("init_it"), erl_int!(6)]
    ]];
    assert_eq!(
        proplist.proplist_get_mfa_string("initial_call"),
        Some("gen_server:init_it/6".to_string())
    );
    assert_eq!(proplist.proplist_get_mfa_string("missing"), None);
}

#[test]
fn test_proplist_get_mfa_string_or() {
    let proplist = erl_list![erl_tuple![
        erl_atom!("current_function"),
        erl_tuple![erl_atom!("erlang"), erl_atom!("hibernate"), erl_int!(3)]
    ]];
    assert_eq!(
        proplist.proplist_get_mfa_string_or("current_function", "unknown"),
        "erlang:hibernate/3".to_string()
    );
    assert_eq!(
        proplist.proplist_get_mfa_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_proplist_get_i64_or() {
    let proplist = erl_list![erl_tuple![erl_atom!("memory"), erl_int!(1024)]];
    assert_eq!(proplist.proplist_get_i64_or("memory", 0), 1024);
    assert_eq!(proplist.proplist_get_i64_or("missing", 0), 0);
}

#[test]
fn test_proplist_get_bool_or() {
    let proplist = erl_list![erl_tuple![erl_atom!("trap_exit"), erl_atom!("true")]];
    assert!(proplist.proplist_get_bool_or("trap_exit", false));
    assert!(!proplist.proplist_get_bool_or("missing", false));
}

#[test]
fn test_proplist_get_string_or() {
    let proplist = erl_list![erl_tuple![
        erl_atom!("name"),
        erl_list![
            erl_int!(65),
            erl_int!(108),
            erl_int!(105),
            erl_int!(99),
            erl_int!(101)
        ]
    ]];
    assert_eq!(
        proplist.proplist_get_string_or("name", "unknown"),
        "Alice".to_string()
    );
    assert_eq!(
        proplist.proplist_get_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_as_erlang_string_or() {
    let term = OwnedTerm::charlist("hello");
    assert_eq!(term.as_erlang_string_or("default"), "hello".to_string());
    assert_eq!(
        erl_int!(42).as_erlang_string_or("default"),
        "default".to_string()
    );
}

#[test]
fn test_as_erlang_string_or_binary() {
    let term = OwnedTerm::Binary(b"world".to_vec());
    assert_eq!(term.as_erlang_string_or("default"), "world".to_string());
}

#[test]
fn test_tuple_get() {
    let tuple = erl_tuple![erl_atom!("ok"), erl_int!(42)];
    assert_eq!(tuple.tuple_get(0), Some(&erl_atom!("ok")));
    assert_eq!(tuple.tuple_get(1), Some(&erl_int!(42)));
    assert_eq!(tuple.tuple_get(2), None);
}

#[test]
fn test_tuple_get_on_non_tuple() {
    assert_eq!(erl_int!(42).tuple_get(0), None);
    assert_eq!(erl_list![].tuple_get(0), None);
}

#[test]
fn test_tuple_get_string() {
    let tuple = erl_tuple![
        erl_atom!("app"),
        OwnedTerm::charlist("description"),
        OwnedTerm::Binary(b"1.0.0".to_vec())
    ];
    assert_eq!(tuple.tuple_get_string(1), Some("description".to_string()));
    assert_eq!(tuple.tuple_get_string(2), Some("1.0.0".to_string()));
}

#[test]
fn test_tuple_get_string_or() {
    let tuple = erl_tuple![erl_atom!("app"), OwnedTerm::charlist("description")];
    assert_eq!(
        tuple.tuple_get_string_or(1, "unknown"),
        "description".to_string()
    );
    assert_eq!(
        tuple.tuple_get_string_or(5, "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_tuple_get_atom_string() {
    let tuple = erl_tuple![erl_atom!("kernel"), erl_int!(42)];
    assert_eq!(tuple.tuple_get_atom_string(0), Some("kernel".to_string()));
    assert_eq!(tuple.tuple_get_atom_string(1), None);
}

#[test]
fn test_tuple_get_atom_string_or() {
    let tuple = erl_tuple![erl_atom!("stdlib")];
    assert_eq!(
        tuple.tuple_get_atom_string_or(0, "unknown"),
        "stdlib".to_string()
    );
    assert_eq!(
        tuple.tuple_get_atom_string_or(1, "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_tuple_get_combined_for_app_info() {
    let app_tuple = erl_tuple![
        erl_atom!("kernel"),
        OwnedTerm::charlist("ERTS  CXC 138 10"),
        OwnedTerm::charlist("9.0")
    ];
    assert_eq!(
        app_tuple.tuple_get_atom_string_or(0, "unknown"),
        "kernel".to_string()
    );
    assert_eq!(
        app_tuple.tuple_get_string_or(1, ""),
        "ERTS  CXC 138 10".to_string()
    );
    assert_eq!(app_tuple.tuple_get_string_or(2, ""), "9.0".to_string());
}

#[test]
fn test_erl_atom_macro() {
    let term = erl_atom!("hello");
    assert_eq!(term, erl_atom!("hello"));
}

#[test]
fn test_erl_atoms_macro() {
    let term = erl_atoms!["a", "b", "c"];
    assert_eq!(
        term,
        erl_list![erl_atom!("a"), erl_atom!("b"), erl_atom!("c")]
    );
}

#[test]
fn test_erl_atoms_macro_empty() {
    let term = erl_atoms![];
    assert_eq!(term, erl_list![]);
}

#[test]
fn test_erl_int_macro() {
    let term = erl_int!(42);
    assert_eq!(term, erl_int!(42));
}

#[test]
fn test_erl_int_macro_negative() {
    let term = erl_int!(-100);
    assert_eq!(term, erl_int!(-100));
}

#[test]
fn test_erl_tuple_macro() {
    let term = erl_tuple![erl_atom!("ok"), erl_int!(42)];
    assert_eq!(term, erl_tuple![erl_atom!("ok"), erl_int!(42)]);
}

#[test]
fn test_erl_tuple_macro_empty() {
    let term = erl_tuple![];
    assert_eq!(term, erl_tuple![]);
}

#[test]
fn test_erl_list_macro() {
    let term = erl_list![erl_int!(1), erl_int!(2), erl_int!(3)];
    assert_eq!(term, erl_list![erl_int!(1), erl_int!(2), erl_int!(3)]);
}

#[test]
fn test_erl_list_macro_empty() {
    let term = erl_list![];
    assert_eq!(term, erl_list![]);
}

#[test]
fn test_erl_map_macro() {
    let term = erl_map! { erl_atom!("key") => erl_int!(42) };
    assert_eq!(term, erl_map! { erl_atom!("key") => erl_int!(42) });
}

#[test]
fn test_erl_map_macro_empty() {
    let term = erl_map! {};
    assert_eq!(term, erl_map! {});
}

#[test]
fn test_erl_macros_combined() {
    let term = erl_tuple![
        erl_atom!("reply"),
        erl_list![erl_int!(1), erl_int!(2)],
        erl_atoms!["a", "b"]
    ];
    assert_eq!(
        term,
        erl_tuple![
            erl_atom!("reply"),
            erl_list![erl_int!(1), erl_int!(2)],
            erl_list![erl_atom!("a"), erl_atom!("b")]
        ]
    );
}

#[test]
fn test_is_charlist_rejects_surrogates() {
    assert!(!erl_list![erl_int!(0xD800)].is_charlist());
    assert!(!erl_list![erl_int!(0xDFFF)].is_charlist());
    assert!(erl_list![erl_int!(0xD7FF)].is_charlist());
    assert!(erl_list![erl_int!(0xE000)].is_charlist());
}

#[test]
fn test_kv_get_on_map() {
    let map = erl_map! {
        erl_atom!("name") => erl_atom!("alice"),
        erl_atom!("age") => erl_int!(30)
    };
    assert_eq!(map.kv_get("name"), Some(&erl_atom!("alice")));
    assert_eq!(map.kv_get("age"), Some(&erl_int!(30)));
    assert_eq!(map.kv_get("missing"), None);
}

#[test]
fn test_kv_get_on_proplist() {
    let proplist = erl_list![
        erl_tuple![erl_atom!("name"), erl_atom!("bob")],
        erl_tuple![erl_atom!("age"), erl_int!(25)]
    ];
    assert_eq!(proplist.kv_get("name"), Some(&erl_atom!("bob")));
    assert_eq!(proplist.kv_get("age"), Some(&erl_int!(25)));
    assert_eq!(proplist.kv_get("missing"), None);
}

#[test]
fn test_kv_get_on_non_container() {
    assert_eq!(erl_int!(42).kv_get("key"), None);
    assert_eq!(erl_atom!("hello").kv_get("key"), None);
    assert_eq!(OwnedTerm::Nil.kv_get("key"), None);
}

#[test]
fn test_kv_get_i64() {
    let map = erl_map! { erl_atom!("count") => erl_int!(42) };
    assert_eq!(map.kv_get_i64("count"), Some(42));
    assert_eq!(map.kv_get_i64("missing"), None);

    let proplist = erl_list![erl_tuple![erl_atom!("count"), erl_int!(100)]];
    assert_eq!(proplist.kv_get_i64("count"), Some(100));
}

#[test]
fn test_kv_get_i64_or() {
    let map = erl_map! { erl_atom!("count") => erl_int!(42) };
    assert_eq!(map.kv_get_i64_or("count", 0), 42);
    assert_eq!(map.kv_get_i64_or("missing", -1), -1);
}

#[test]
fn test_kv_get_bool() {
    let map = erl_map! {
        erl_atom!("enabled") => erl_atom!("true"),
        erl_atom!("disabled") => erl_atom!("false")
    };
    assert_eq!(map.kv_get_bool("enabled"), Some(true));
    assert_eq!(map.kv_get_bool("disabled"), Some(false));
    assert_eq!(map.kv_get_bool("missing"), None);
}

#[test]
fn test_kv_get_bool_or() {
    let map = erl_map! { erl_atom!("flag") => erl_atom!("true") };
    assert!(map.kv_get_bool_or("flag", false));
    assert!(!map.kv_get_bool_or("missing", false));
}

#[test]
fn test_kv_get_atom() {
    let map = erl_map! { erl_atom!("status") => erl_atom!("running") };
    assert_eq!(map.kv_get_atom("status"), Some(&Atom::new("running")));
    assert_eq!(map.kv_get_atom("missing"), None);
}

#[test]
fn test_kv_get_atom_string() {
    let map = erl_map! { erl_atom!("status") => erl_atom!("running") };
    assert_eq!(
        map.kv_get_atom_string("status"),
        Some("running".to_string())
    );
    assert_eq!(map.kv_get_atom_string("missing"), None);
}

#[test]
fn test_kv_get_atom_string_or() {
    let map = erl_map! { erl_atom!("status") => erl_atom!("running") };
    assert_eq!(
        map.kv_get_atom_string_or("status", "unknown"),
        "running".to_string()
    );
    assert_eq!(
        map.kv_get_atom_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_kv_get_string() {
    let map = erl_map! { erl_atom!("name") => OwnedTerm::charlist("Alice") };
    assert_eq!(map.kv_get_string("name"), Some("Alice".to_string()));
    assert_eq!(map.kv_get_string("missing"), None);
}

#[test]
fn test_kv_get_string_or() {
    let map = erl_map! { erl_atom!("name") => OwnedTerm::charlist("Alice") };
    assert_eq!(map.kv_get_string_or("name", "unknown"), "Alice".to_string());
    assert_eq!(
        map.kv_get_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_kv_get_mfa_string() {
    let mfa = erl_tuple![erl_atom!("erlang"), erl_atom!("apply"), erl_int!(2)];
    let map = erl_map! { erl_atom!("current_function") => mfa };
    assert_eq!(
        map.kv_get_mfa_string("current_function"),
        Some("erlang:apply/2".to_string())
    );
    assert_eq!(map.kv_get_mfa_string("missing"), None);
}

#[test]
fn test_kv_get_mfa_string_or() {
    let mfa = erl_tuple![erl_atom!("gen_server"), erl_atom!("loop"), erl_int!(7)];
    let map = erl_map! { erl_atom!("initial_call") => mfa };
    assert_eq!(
        map.kv_get_mfa_string_or("initial_call", "unknown"),
        "gen_server:loop/7".to_string()
    );
    assert_eq!(
        map.kv_get_mfa_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_map_get_i64() {
    let map = erl_map! { erl_atom!("count") => erl_int!(42) };
    assert_eq!(map.map_get_i64("count"), Some(42));
    assert_eq!(map.map_get_i64("missing"), None);
}

#[test]
fn test_map_get_i64_or() {
    let map = erl_map! { erl_atom!("count") => erl_int!(42) };
    assert_eq!(map.map_get_i64_or("count", 0), 42);
    assert_eq!(map.map_get_i64_or("missing", -1), -1);
}

#[test]
fn test_map_get_bool() {
    let map = erl_map! {
        erl_atom!("enabled") => erl_atom!("true"),
        erl_atom!("disabled") => erl_atom!("false")
    };
    assert_eq!(map.map_get_bool("enabled"), Some(true));
    assert_eq!(map.map_get_bool("disabled"), Some(false));
    assert_eq!(map.map_get_bool("missing"), None);
}

#[test]
fn test_map_get_bool_or() {
    let map = erl_map! { erl_atom!("flag") => erl_atom!("true") };
    assert!(map.map_get_bool_or("flag", false));
    assert!(!map.map_get_bool_or("missing", false));
}

#[test]
fn test_map_get_atom() {
    let map = erl_map! { erl_atom!("status") => erl_atom!("running") };
    assert_eq!(map.map_get_atom("status"), Some(&Atom::new("running")));
    assert_eq!(map.map_get_atom("missing"), None);
}

#[test]
fn test_map_get_string() {
    let map = erl_map! { erl_atom!("name") => OwnedTerm::charlist("Alice") };
    assert_eq!(map.map_get_string("name"), Some("Alice".to_string()));
    assert_eq!(map.map_get_string("missing"), None);
}

#[test]
fn test_map_get_string_or() {
    let map = erl_map! { erl_atom!("name") => OwnedTerm::charlist("Alice") };
    assert_eq!(
        map.map_get_string_or("name", "unknown"),
        "Alice".to_string()
    );
    assert_eq!(
        map.map_get_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_map_get_atom_string() {
    let map = erl_map! { erl_atom!("status") => erl_atom!("running") };
    assert_eq!(
        map.map_get_atom_string("status"),
        Some("running".to_string())
    );
    assert_eq!(map.map_get_atom_string("missing"), None);
}

#[test]
fn test_map_get_atom_string_or() {
    let map = erl_map! { erl_atom!("status") => erl_atom!("running") };
    assert_eq!(
        map.map_get_atom_string_or("status", "unknown"),
        "running".to_string()
    );
    assert_eq!(
        map.map_get_atom_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_map_get_mfa_string() {
    let mfa = erl_tuple![erl_atom!("erlang"), erl_atom!("apply"), erl_int!(2)];
    let map = erl_map! { erl_atom!("current_function") => mfa };
    assert_eq!(
        map.map_get_mfa_string("current_function"),
        Some("erlang:apply/2".to_string())
    );
    assert_eq!(map.map_get_mfa_string("missing"), None);
}

#[test]
fn test_map_get_mfa_string_or() {
    let mfa = erl_tuple![erl_atom!("gen_server"), erl_atom!("loop"), erl_int!(7)];
    let map = erl_map! { erl_atom!("initial_call") => mfa };
    assert_eq!(
        map.map_get_mfa_string_or("initial_call", "unknown"),
        "gen_server:loop/7".to_string()
    );
    assert_eq!(
        map.map_get_mfa_string_or("missing", "unknown"),
        "unknown".to_string()
    );
}

#[test]
fn test_kv_get_i64_on_map_and_proplist() {
    let map = erl_map! { erl_atom!("count") => erl_int!(42) };
    assert_eq!(map.kv_get_i64("count"), Some(42));
    assert_eq!(map.kv_get_i64("missing"), None);

    let proplist = erl_list![erl_tuple![erl_atom!("count"), erl_int!(100)]];
    assert_eq!(proplist.kv_get_i64("count"), Some(100));
    assert_eq!(proplist.kv_get_i64("missing"), None);
}

#[test]
fn test_kv_get_i64_or_on_map_and_proplist() {
    let map = erl_map! { erl_atom!("count") => erl_int!(42) };
    assert_eq!(map.kv_get_i64_or("count", 0), 42);
    assert_eq!(map.kv_get_i64_or("missing", -1), -1);

    let proplist = erl_list![erl_tuple![erl_atom!("count"), erl_int!(100)]];
    assert_eq!(proplist.kv_get_i64_or("count", 0), 100);
    assert_eq!(proplist.kv_get_i64_or("missing", -1), -1);
}

#[test]
fn test_map_get_i64_variations() {
    let map = erl_map! { erl_atom!("a") => erl_int!(1), erl_atom!("b") => erl_int!(2), erl_atom!("c") => erl_int!(3) };
    assert_eq!(map.map_get_i64("a"), Some(1));
    assert_eq!(map.map_get_i64("b"), Some(2));
    assert_eq!(map.map_get_i64("c"), Some(3));
    assert_eq!(map.map_get_i64("d"), None);
}

#[test]
fn test_map_get_i64_or_variations() {
    let map = erl_map! { erl_atom!("x") => erl_int!(10), erl_atom!("y") => erl_int!(20) };
    assert_eq!(map.map_get_i64_or("x", 0), 10);
    assert_eq!(map.map_get_i64_or("y", 0), 20);
    assert_eq!(map.map_get_i64_or("z", 99), 99);
}

#[test]
fn test_proplist_get_i64_or_variations() {
    let proplist = erl_list![
        erl_tuple![erl_atom!("x"), erl_int!(10)],
        erl_tuple![erl_atom!("y"), erl_int!(20)]
    ];
    assert_eq!(proplist.proplist_get_i64_or("x", 0), 10);
    assert_eq!(proplist.proplist_get_i64_or("y", 0), 20);
    assert_eq!(proplist.proplist_get_i64_or("z", 99), 99);
}
