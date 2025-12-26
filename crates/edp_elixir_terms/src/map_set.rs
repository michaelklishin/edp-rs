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

//! Elixir MapSet type support.
//!
//! Uses the `:sets` v2 format (Elixir 1.17+).

use erltf::{Atom, OwnedTerm};
use std::collections::{BTreeMap, BTreeSet};

/// Represents an Elixir MapSet.
///
/// MapSet is a set data structure backed by a map where each element
/// is a key with an empty list as value.
///
/// # Example
///
/// ```
/// use edp_elixir_terms::ElixirMapSet;
/// use erltf::OwnedTerm;
///
/// let mut set = ElixirMapSet::new();
/// set.insert(OwnedTerm::integer(1));
/// set.insert(OwnedTerm::integer(2));
/// set.insert(OwnedTerm::integer(3));
///
/// let term: OwnedTerm = set.into();
/// assert!(term.is_elixir_struct());
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ElixirMapSet {
    elements: BTreeSet<OwnedTerm>,
}

impl ElixirMapSet {
    /// Creates a new empty MapSet.
    #[must_use]
    pub fn new() -> Self {
        Self {
            elements: BTreeSet::new(),
        }
    }

    /// Creates a MapSet from values that convert to OwnedTerm.
    #[must_use]
    pub fn from_values<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OwnedTerm>,
    {
        Self {
            elements: iter.into_iter().map(Into::into).collect(),
        }
    }

    /// Inserts a value into the set.
    pub fn insert<T: Into<OwnedTerm>>(&mut self, value: T) -> bool {
        self.elements.insert(value.into())
    }

    /// Removes a value from the set.
    pub fn remove(&mut self, value: &OwnedTerm) -> bool {
        self.elements.remove(value)
    }

    /// Removes all elements from the set.
    pub fn clear(&mut self) {
        self.elements.clear();
    }

    /// Returns true if the set contains the value.
    #[must_use]
    pub fn contains(&self, value: &OwnedTerm) -> bool {
        self.elements.contains(value)
    }

    /// Returns the number of elements in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Returns true if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> impl Iterator<Item = &OwnedTerm> {
        self.elements.iter()
    }

    /// Returns the union of this set with another.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            elements: self.elements.union(&other.elements).cloned().collect(),
        }
    }

    /// Returns the intersection of this set with another.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            elements: self
                .elements
                .intersection(&other.elements)
                .cloned()
                .collect(),
        }
    }

    /// Returns the difference of this set with another.
    #[must_use]
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            elements: self.elements.difference(&other.elements).cloned().collect(),
        }
    }

    /// Returns the symmetric difference of this set with another.
    #[must_use]
    pub fn symmetric_difference(&self, other: &Self) -> Self {
        Self {
            elements: self
                .elements
                .symmetric_difference(&other.elements)
                .cloned()
                .collect(),
        }
    }

    /// Returns true if this set is a subset of another.
    #[must_use]
    pub fn is_subset(&self, other: &Self) -> bool {
        self.elements.is_subset(&other.elements)
    }

    /// Returns true if this set is a superset of another.
    #[must_use]
    pub fn is_superset(&self, other: &Self) -> bool {
        self.elements.is_superset(&other.elements)
    }

    /// Returns true if this set has no elements in common with another.
    #[must_use]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.elements.is_disjoint(&other.elements)
    }

    /// Parses an OwnedTerm as a MapSet struct.
    ///
    /// Expects `:sets` v2 format: `{:set, size, %{elem => []}}`.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.MapSet") {
            return None;
        }

        let map = term.as_map()?;
        let map_key = OwnedTerm::Atom(Atom::new("map"));
        let map_value = map.get(&map_key)?;

        // :sets v2 format: {:set, size, %{elem => []}}
        let tuple = map_value.as_tuple()?;
        if tuple.len() != 3 || tuple[0].atom_name() != Some("set") {
            return None;
        }

        let inner_map = tuple[2].as_map()?;
        let elements: BTreeSet<OwnedTerm> = inner_map.keys().cloned().collect();
        Some(Self { elements })
    }
}

impl From<ElixirMapSet> for OwnedTerm {
    fn from(set: ElixirMapSet) -> Self {
        // Elixir 1.17+ MapSet uses :sets v2:
        // %MapSet{map: {:set, size, %{elem1 => [], elem2 => [], ...}}}
        let size = set.elements.len() as i64;
        let inner_map: BTreeMap<OwnedTerm, OwnedTerm> = set
            .elements
            .into_iter()
            .map(|elem| (elem, OwnedTerm::List(vec![])))
            .collect();

        // Build the :sets v2 tuple: {:set, size, map}
        let sets_tuple = OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("set")),
            OwnedTerm::Integer(size),
            OwnedTerm::Map(inner_map),
        ]);

        let mut outer_map = BTreeMap::new();
        outer_map.insert(
            OwnedTerm::Atom(Atom::new("__struct__")),
            OwnedTerm::Atom(Atom::new("Elixir.MapSet")),
        );
        outer_map.insert(OwnedTerm::Atom(Atom::new("map")), sets_tuple);

        OwnedTerm::Map(outer_map)
    }
}

impl FromIterator<OwnedTerm> for ElixirMapSet {
    fn from_iter<I: IntoIterator<Item = OwnedTerm>>(iter: I) -> Self {
        Self {
            elements: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for ElixirMapSet {
    type Item = OwnedTerm;
    type IntoIter = std::collections::btree_set::IntoIter<OwnedTerm>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for &'a ElixirMapSet {
    type Item = &'a OwnedTerm;
    type IntoIter = std::collections::btree_set::Iter<'a, OwnedTerm>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}
