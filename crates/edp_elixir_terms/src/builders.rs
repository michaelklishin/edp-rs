// Copyright (C) 2025 Michael S. Klishin and Contributors
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

//! Type-safe builders for Elixir data structures.

use erltf::{Atom, OwnedTerm};
use std::collections::BTreeMap;

/// Builder for Elixir keyword lists (lists of 2-tuples with atom keys).
///
/// Keyword lists preserve insertion order and allow duplicate keys.
///
/// # Example
///
/// ```
/// use edp_elixir_terms::KeywordListBuilder;
///
/// let kw = KeywordListBuilder::new()
///     .put("timeout", 5000)
///     .put("retry", true)
///     .put("name", "worker")
///     .build();
///
/// assert!(kw.is_proplist());
/// ```
#[derive(Debug, Clone, Default)]
pub struct KeywordListBuilder {
    elements: Vec<OwnedTerm>,
}

impl KeywordListBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Creates a builder with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
        }
    }

    /// Appends a key-value pair.
    pub fn put<V: Into<OwnedTerm>>(mut self, key: &str, value: V) -> Self {
        self.elements.push(OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new(key)),
            value.into(),
        ]));
        self
    }

    /// Appends a key-value pair where the value is also an atom.
    pub fn put_atom(mut self, key: &str, value: &str) -> Self {
        self.elements.push(OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new(key)),
            OwnedTerm::Atom(Atom::new(value)),
        ]));
        self
    }

    /// Appends `{key, true}` to the keyword list.
    pub fn put_flag(mut self, key: &str) -> Self {
        self.elements.push(OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new(key)),
            OwnedTerm::boolean(true),
        ]));
        self
    }

    /// Appends a raw OwnedTerm value.
    pub fn put_term(mut self, key: &str, value: OwnedTerm) -> Self {
        self.elements.push(OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new(key)),
            value,
        ]));
        self
    }

    /// Conditionally appends a key-value pair.
    pub fn put_if<V: Into<OwnedTerm>>(self, condition: bool, key: &str, value: V) -> Self {
        if condition {
            self.put(key, value)
        } else {
            self
        }
    }

    /// Appends a key-value pair only if the Option is Some.
    pub fn put_some<V: Into<OwnedTerm>>(self, key: &str, value: Option<V>) -> Self {
        match value {
            Some(v) => self.put(key, v),
            None => self,
        }
    }

    /// Extends the keyword list with another iterable of key-value pairs.
    pub fn extend<I, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (&'static str, V)>,
        V: Into<OwnedTerm>,
    {
        for (key, value) in iter {
            self.elements.push(OwnedTerm::Tuple(vec![
                OwnedTerm::Atom(Atom::new(key)),
                value.into(),
            ]));
        }
        self
    }

    /// Returns the number of key-value pairs in the builder.
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns true if the builder is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Builds the keyword list as an OwnedTerm.
    #[must_use]
    pub fn build(self) -> OwnedTerm {
        OwnedTerm::List(self.elements)
    }
}

/// Builder for Elixir maps with atom keys.
///
/// Unlike keyword lists, maps do not allow duplicate keys and have
/// no guaranteed ordering.
///
/// # Example
///
/// ```
/// use edp_elixir_terms::AtomKeyMapBuilder;
///
/// let map = AtomKeyMapBuilder::new()
///     .insert("name", "Alice")
///     .insert("age", 30)
///     .insert("active", true)
///     .build();
///
/// assert!(map.is_map());
/// ```
#[derive(Debug, Clone, Default)]
pub struct AtomKeyMapBuilder {
    map: BTreeMap<OwnedTerm, OwnedTerm>,
}

impl AtomKeyMapBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    /// Inserts a key-value pair. Replaces existing values.
    pub fn insert<V: Into<OwnedTerm>>(mut self, key: &str, value: V) -> Self {
        self.map
            .insert(OwnedTerm::Atom(Atom::new(key)), value.into());
        self
    }

    /// Inserts a key-value pair where the value is also an atom.
    pub fn insert_atom(mut self, key: &str, value: &str) -> Self {
        self.map.insert(
            OwnedTerm::Atom(Atom::new(key)),
            OwnedTerm::Atom(Atom::new(value)),
        );
        self
    }

    /// Inserts a raw OwnedTerm value.
    pub fn insert_term(mut self, key: &str, value: OwnedTerm) -> Self {
        self.map.insert(OwnedTerm::Atom(Atom::new(key)), value);
        self
    }

    /// Conditionally inserts a key-value pair.
    pub fn insert_if<V: Into<OwnedTerm>>(self, condition: bool, key: &str, value: V) -> Self {
        if condition {
            self.insert(key, value)
        } else {
            self
        }
    }

    /// Inserts a key-value pair only if the Option is Some.
    pub fn insert_some<V: Into<OwnedTerm>>(self, key: &str, value: Option<V>) -> Self {
        match value {
            Some(v) => self.insert(key, v),
            None => self,
        }
    }

    /// Extends the map with another iterable of key-value pairs.
    pub fn extend<I, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (&'static str, V)>,
        V: Into<OwnedTerm>,
    {
        for (key, value) in iter {
            self.map
                .insert(OwnedTerm::Atom(Atom::new(key)), value.into());
        }
        self
    }

    /// Returns the number of key-value pairs in the builder.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the builder is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Builds the map as an OwnedTerm.
    #[must_use]
    pub fn build(self) -> OwnedTerm {
        OwnedTerm::Map(self.map)
    }

    /// Builds an Elixir struct. The "Elixir." prefix is added automatically.
    #[must_use]
    pub fn build_struct(mut self, module: &str) -> OwnedTerm {
        let full_module = format!("Elixir.{module}");
        self.map.insert(
            OwnedTerm::Atom(Atom::new("__struct__")),
            OwnedTerm::Atom(Atom::new(&full_module)),
        );
        OwnedTerm::Map(self.map)
    }
}
