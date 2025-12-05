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

//! Support for Elixir struct serialization.
//!
//! Elixir structs are Erlang maps with a special `__struct__` key containing
//! the module name as an atom. This module provides helper types to serialize
//! Rust structs in this format.
//!
//! # Example
//!
//! ```ignore
//! use erltf_serde::ElixirStruct;
//!
//! #[derive(ElixirStruct)]
//! #[elixir_module = "MyApp.User"]
//! struct User {
//!     name: String,
//!     age: i32,
//! }
//! ```

use serde::Serialize;

pub const ATOM_KEY_MARKER: &str = "__erltf_atom_key__";
pub const ATOM_VALUE_MARKER: &str = "__erltf_atom_value__";

/// Wrapper type that signals the serializer to emit an atom key.
///
/// Used by the `ElixirStruct` derive macro to serialize struct field names
/// as atoms instead of binaries.
#[derive(Debug, Clone, Copy)]
pub struct AtomKey<'a>(pub &'a str);

impl Serialize for AtomKey<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(ATOM_KEY_MARKER, self.0)
    }
}

/// Wrapper type that signals the serializer to emit an atom value.
///
/// Used by the `ElixirStruct` derive macro to serialize the `__struct__`
/// field value as an atom.
#[derive(Debug, Clone, Copy)]
pub struct AtomValue<'a>(pub &'a str);

impl Serialize for AtomValue<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(ATOM_VALUE_MARKER, self.0)
    }
}
