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
pub use term::OwnedTerm;
pub use types::{Atom, BigInt, ExternalPid, ExternalPort, ExternalReference, Sign};

#[macro_export]
macro_rules! term_tuple {
    ($($elem:expr),* $(,)?) => {
        $crate::OwnedTerm::Tuple(vec![$($elem.into()),*])
    };
}

#[macro_export]
macro_rules! term_list {
    ($($elem:expr),* $(,)?) => {
        $crate::OwnedTerm::List(vec![$($elem.into()),*])
    };
}

#[macro_export]
macro_rules! term_map {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::BTreeMap::new();
        $(
            map.insert($key.into(), $value.into());
        )*
        $crate::OwnedTerm::Map(map)
    }};
}
