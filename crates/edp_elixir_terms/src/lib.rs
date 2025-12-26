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

//! Elixir term construction helpers for edp-rs.
//!
//! Helpers for constructing Elixir-compatible terms. These are building blocks
//! for easier Rust-Elixir interop via EDP.
//!
//! # Example
//!
//! ```
//! use edp_elixir_terms::{KeywordListBuilder, AtomKeyMapBuilder, ElixirRange};
//!
//! // Build a keyword list
//! let kw = KeywordListBuilder::new()
//!     .put("name", "Alice")
//!     .put("age", 30)
//!     .build();
//!
//! // Build an atom-keyed map
//! let map = AtomKeyMapBuilder::new()
//!     .insert("status", "active")
//!     .insert("count", 42)
//!     .build();
//!
//! // Create an Elixir Range
//! let range = ElixirRange::new(1, 10, 1);
//! ```

mod builders;
mod date_time;
mod exceptions;
mod gen_server_terms;
mod map_set;
mod range;

pub use builders::{AtomKeyMapBuilder, KeywordListBuilder};
pub use date_time::{ElixirDate, ElixirDateTime, ElixirNaiveDateTime, ElixirTime};
pub use exceptions::{
    ArgumentError, ArithmeticError, BadFunctionError, BadMapError, CaseClauseError,
    CondClauseError, ElixirExceptionExt, FunctionClauseError, KeyError, MatchError, RuntimeError,
    UndefinedFunctionError, WithClauseError,
};
pub use gen_server_terms::GenServerTerms;
pub use map_set::ElixirMapSet;
pub use range::{ElixirRange, RangeIterator};
