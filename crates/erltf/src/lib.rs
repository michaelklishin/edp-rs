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

pub mod borrowed;
pub mod decoder;
pub mod encoder;
pub mod errors;
pub mod tags;
pub mod term;
pub mod types;

pub use borrowed::BorrowedTerm;
pub use decoder::{AtomCache, decode, decode_borrowed, decode_with_atom_cache};
pub use encoder::{
    encode, encode_to_writer, encode_with_dist_header, encode_with_dist_header_multi,
};
pub use errors::{
    ContextualDecodeError, DecodeError, EncodeError, Error, ParsingContext, PathSegment, Result,
};
pub use term::{KeyValueAccess, OwnedTerm};
pub use types::{Atom, BigInt, ExternalPid, ExternalPort, ExternalReference, Mfa, Sign};

#[macro_export]
macro_rules! erl_tuple {
    ($($elem:expr),* $(,)?) => {
        $crate::OwnedTerm::Tuple(vec![$($elem.into()),*])
    };
}

#[macro_export]
macro_rules! erl_list {
    ($($elem:expr),* $(,)?) => {
        $crate::OwnedTerm::List(vec![$($elem.into()),*])
    };
}

#[macro_export]
macro_rules! erl_map {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::BTreeMap::new();
        $(
            map.insert($key.into(), $value.into());
        )*
        $crate::OwnedTerm::Map(map)
    }};
}

#[macro_export]
macro_rules! erl_atom {
    ($name:expr) => {
        $crate::OwnedTerm::Atom($crate::Atom::new($name))
    };
}

#[macro_export]
macro_rules! erl_atoms {
    ($($name:expr),* $(,)?) => {
        $crate::OwnedTerm::List(vec![$($crate::OwnedTerm::Atom($crate::Atom::new($name))),*])
    };
}

#[macro_export]
macro_rules! erl_int {
    ($val:expr) => {
        $crate::OwnedTerm::Integer($val as i64)
    };
}

/// Creates an Elixir-style keyword list (list of 2-tuples with atom keys).
///
/// # Example
/// ```
/// use erltf::keyword;
///
/// let kw = keyword![
///     name: "Alice",
///     age: 30,
///     active: true,
/// ];
///
/// assert!(kw.is_proplist());
/// assert_eq!(kw.proplist_get_atom_key("name").and_then(|t| t.as_erlang_string()), Some("Alice".to_string()));
/// ```
#[macro_export]
macro_rules! keyword {
    ($($key:ident : $value:expr),* $(,)?) => {{
        $crate::OwnedTerm::List(vec![
            $(
                $crate::OwnedTerm::Tuple(vec![
                    $crate::OwnedTerm::Atom($crate::Atom::new(stringify!($key))),
                    $value.into()
                ])
            ),*
        ])
    }};
}

/// Creates an Elixir module atom (automatically prefixes with "Elixir.").
///
/// # Example
/// ```
/// use erltf::elixir_module;
///
/// let module = elixir_module!("MyApp.Users.User");
/// assert!(module.is_atom_with_name("Elixir.MyApp.Users.User"));
/// ```
#[macro_export]
macro_rules! elixir_module {
    ($name:expr) => {
        $crate::OwnedTerm::Atom($crate::Atom::new(&format!("Elixir.{}", $name)))
    };
}
