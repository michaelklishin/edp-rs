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

//! Serde integration for the Erlang Term Format.
//!
//! # Features
//!
//! ## `elixir-interop`
//!
//! Enables Elixir conventions for `Option` types:
//!
//! * `Option::None` serializes as `nil` (Elixir) instead of `undefined` (Erlang)
//! * Both `nil` and `undefined` deserialize as `Option::None`

mod de;
pub mod elixir;
mod error;
mod ser;

pub use de::{Deserializer, ProplistDeserializer, from_bytes, from_proplist, from_term};
pub use erltf_serde_derive::ElixirStruct;
pub use error::{Error, Result};
pub use ser::{Serializer, to_bytes, to_term};

use erltf::OwnedTerm;
use serde::de::DeserializeOwned;

pub trait OwnedTermExt {
    fn try_deserialize<T: DeserializeOwned>(&self) -> Result<T>;

    fn try_deserialize_proplist<T: DeserializeOwned>(&self) -> Result<T>;
}

impl OwnedTermExt for OwnedTerm {
    fn try_deserialize<T: DeserializeOwned>(&self) -> Result<T> {
        let normalized = self
            .to_map_recursive()
            .map_err(|e| Error::Message(e.to_string()))?;
        from_term(&normalized)
    }

    fn try_deserialize_proplist<T: DeserializeOwned>(&self) -> Result<T> {
        from_proplist(self)
    }
}
