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

//! Elixir Range type support.

use erltf::{Atom, OwnedTerm};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents an Elixir Range (`first..last//step`).
///
/// Elixir ranges are inclusive sequences of integers with a step value.
///
/// # Example
///
/// ```
/// use edp_elixir_terms::ElixirRange;
/// use erltf::OwnedTerm;
///
/// // Create a range 1..10 with step 1
/// let range = ElixirRange::new(1, 10, 1);
/// let term: OwnedTerm = range.into();
///
/// assert!(term.is_elixir_struct());
/// assert_eq!(term.elixir_struct_module(), Some("Elixir.Range"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElixirRange {
    pub first: i64,
    pub last: i64,
    pub step: i64,
}

impl ElixirRange {
    /// Creates a new Range.
    #[must_use]
    pub fn new(first: i64, last: i64, step: i64) -> Self {
        Self { first, last, step }
    }

    /// Creates an ascending range with step 1.
    #[must_use]
    pub fn ascending(first: i64, last: i64) -> Self {
        Self::new(first, last, 1)
    }

    /// Creates a descending range with step -1.
    #[must_use]
    pub fn descending(first: i64, last: i64) -> Self {
        Self::new(first, last, -1)
    }

    /// Returns true if this is an empty range.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        if self.step > 0 {
            self.first > self.last
        } else if self.step < 0 {
            self.first < self.last
        } else {
            true
        }
    }

    /// Returns the number of elements in the range.
    #[must_use]
    pub fn len(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        let diff = (self.last - self.first).abs();
        let step = self.step.abs();
        ((diff / step) + 1) as usize
    }

    /// Returns true if the range contains the given value.
    #[must_use]
    pub fn contains(&self, value: i64) -> bool {
        if self.is_empty() {
            return false;
        }
        if self.step > 0 {
            value >= self.first && value <= self.last && (value - self.first) % self.step == 0
        } else {
            value <= self.first && value >= self.last && (self.first - value) % (-self.step) == 0
        }
    }

    /// Parses an OwnedTerm as a Range struct.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.Range") {
            return None;
        }

        let map = term.as_map()?;
        let first_key = OwnedTerm::Atom(Atom::new("first"));
        let last_key = OwnedTerm::Atom(Atom::new("last"));
        let step_key = OwnedTerm::Atom(Atom::new("step"));

        let first = map.get(&first_key)?.as_integer()?;
        let last = map.get(&last_key)?.as_integer()?;
        let step = map.get(&step_key)?.as_integer()?;

        Some(Self { first, last, step })
    }
}

impl From<ElixirRange> for OwnedTerm {
    fn from(range: ElixirRange) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            OwnedTerm::Atom(Atom::new("__struct__")),
            OwnedTerm::Atom(Atom::new("Elixir.Range")),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("first")),
            OwnedTerm::Integer(range.first),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("last")),
            OwnedTerm::Integer(range.last),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("step")),
            OwnedTerm::Integer(range.step),
        );
        OwnedTerm::Map(map)
    }
}

impl IntoIterator for ElixirRange {
    type Item = i64;
    type IntoIter = RangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        RangeIterator {
            current: self.first,
            range: self,
            done: false,
        }
    }
}

/// Iterator over an ElixirRange.
pub struct RangeIterator {
    current: i64,
    range: ElixirRange,
    done: bool,
}

impl Iterator for RangeIterator {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.range.is_empty() {
            return None;
        }

        let value = self.current;

        if self.range.step > 0 {
            if value > self.range.last {
                self.done = true;
                return None;
            }
            if value == self.range.last {
                self.done = true;
            } else {
                self.current = self.current.saturating_add(self.range.step);
            }
        } else {
            if value < self.range.last {
                self.done = true;
                return None;
            }
            if value == self.range.last {
                self.done = true;
            } else {
                self.current = self.current.saturating_add(self.range.step);
            }
        }

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done || self.range.is_empty() {
            return (0, Some(0));
        }
        let remaining = if self.range.step > 0 {
            if self.current > self.range.last {
                0
            } else {
                (((self.range.last - self.current) / self.range.step) + 1) as usize
            }
        } else if self.current < self.range.last {
            0
        } else {
            (((self.current - self.range.last) / (-self.range.step)) + 1) as usize
        };
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for RangeIterator {}

impl std::fmt::Display for ElixirRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.step == 1 {
            write!(f, "{}..{}", self.first, self.last)
        } else {
            write!(f, "{}..{}//{}", self.first, self.last, self.step)
        }
    }
}
