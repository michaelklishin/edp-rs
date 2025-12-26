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

//! Elixir exception type support.

use erltf::{Atom, OwnedTerm};
use std::collections::BTreeMap;

/// Extension trait for creating Elixir exceptions.
pub trait ElixirExceptionExt {
    /// Returns the Elixir module name for this exception.
    fn module_name() -> &'static str;

    /// Converts this exception to an OwnedTerm.
    fn to_term(&self) -> OwnedTerm;
}

/// Creates the base structure for an Elixir exception.
fn exception_base(module: &str) -> BTreeMap<OwnedTerm, OwnedTerm> {
    let mut map = BTreeMap::new();
    map.insert(
        OwnedTerm::Atom(Atom::new("__struct__")),
        OwnedTerm::Atom(Atom::new(module)),
    );
    map.insert(
        OwnedTerm::Atom(Atom::new("__exception__")),
        OwnedTerm::boolean(true),
    );
    map
}

/// Elixir ArgumentError exception.
///
/// Raised when a function receives an argument with an unexpected value or type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentError {
    pub message: String,
}

impl ArgumentError {
    /// Creates a new ArgumentError.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Parses an OwnedTerm as an ArgumentError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.ArgumentError") {
            return None;
        }
        let map = term.as_map()?;
        let message = map
            .get(&OwnedTerm::Atom(Atom::new("message")))?
            .as_erlang_string()?;
        Some(Self { message })
    }
}

impl ElixirExceptionExt for ArgumentError {
    fn module_name() -> &'static str {
        "Elixir.ArgumentError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(
            OwnedTerm::Atom(Atom::new("message")),
            OwnedTerm::Binary(self.message.clone().into_bytes()),
        );
        OwnedTerm::Map(map)
    }
}

impl From<ArgumentError> for OwnedTerm {
    fn from(err: ArgumentError) -> Self {
        err.to_term()
    }
}

/// Elixir RuntimeError exception.
///
/// Raised for general runtime errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    /// Creates a new RuntimeError.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Parses an OwnedTerm as a RuntimeError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.RuntimeError") {
            return None;
        }
        let map = term.as_map()?;
        let message = map
            .get(&OwnedTerm::Atom(Atom::new("message")))?
            .as_erlang_string()?;
        Some(Self { message })
    }
}

impl ElixirExceptionExt for RuntimeError {
    fn module_name() -> &'static str {
        "Elixir.RuntimeError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(
            OwnedTerm::Atom(Atom::new("message")),
            OwnedTerm::Binary(self.message.clone().into_bytes()),
        );
        OwnedTerm::Map(map)
    }
}

impl From<RuntimeError> for OwnedTerm {
    fn from(err: RuntimeError) -> Self {
        err.to_term()
    }
}

/// Elixir KeyError exception.
///
/// Raised when a key is not found in a map or keyword list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyError {
    pub key: OwnedTerm,
    pub term: OwnedTerm,
    pub message: Option<String>,
}

impl KeyError {
    /// Creates a new KeyError.
    #[must_use]
    pub fn new(key: OwnedTerm, term: OwnedTerm) -> Self {
        Self {
            key,
            term,
            message: None,
        }
    }

    /// Creates a KeyError with a custom message.
    #[must_use]
    pub fn with_message(key: OwnedTerm, term: OwnedTerm, message: impl Into<String>) -> Self {
        Self {
            key,
            term,
            message: Some(message.into()),
        }
    }

    /// Parses an OwnedTerm as a KeyError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.KeyError") {
            return None;
        }
        let map = term.as_map()?;
        let key = map.get(&OwnedTerm::Atom(Atom::new("key")))?.clone();
        let container = map.get(&OwnedTerm::Atom(Atom::new("term")))?.clone();
        let message = map
            .get(&OwnedTerm::Atom(Atom::new("message")))
            .and_then(|v| v.as_erlang_string());
        Some(Self {
            key,
            term: container,
            message,
        })
    }
}

impl ElixirExceptionExt for KeyError {
    fn module_name() -> &'static str {
        "Elixir.KeyError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(OwnedTerm::Atom(Atom::new("key")), self.key.clone());
        map.insert(OwnedTerm::Atom(Atom::new("term")), self.term.clone());
        map.insert(
            OwnedTerm::Atom(Atom::new("message")),
            self.message.as_ref().map_or(OwnedTerm::elixir_nil(), |m| {
                OwnedTerm::Binary(m.clone().into_bytes())
            }),
        );
        OwnedTerm::Map(map)
    }
}

impl From<KeyError> for OwnedTerm {
    fn from(err: KeyError) -> Self {
        err.to_term()
    }
}

/// Elixir MatchError exception.
///
/// Raised when a pattern match fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchError {
    pub term: OwnedTerm,
}

impl MatchError {
    /// Creates a new MatchError.
    #[must_use]
    pub fn new(term: OwnedTerm) -> Self {
        Self { term }
    }

    /// Parses an OwnedTerm as a MatchError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.MatchError") {
            return None;
        }
        let map = term.as_map()?;
        let value = map.get(&OwnedTerm::Atom(Atom::new("term")))?.clone();
        Some(Self { term: value })
    }
}

impl ElixirExceptionExt for MatchError {
    fn module_name() -> &'static str {
        "Elixir.MatchError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(OwnedTerm::Atom(Atom::new("term")), self.term.clone());
        OwnedTerm::Map(map)
    }
}

impl From<MatchError> for OwnedTerm {
    fn from(err: MatchError) -> Self {
        err.to_term()
    }
}

/// Elixir UndefinedFunctionError exception.
///
/// Raised when a function is called but does not exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndefinedFunctionError {
    pub module: String,
    pub function: String,
    pub arity: u8,
    pub reason: Option<String>,
}

impl UndefinedFunctionError {
    /// Creates a new UndefinedFunctionError.
    #[must_use]
    pub fn new(module: impl Into<String>, function: impl Into<String>, arity: u8) -> Self {
        Self {
            module: module.into(),
            function: function.into(),
            arity,
            reason: None,
        }
    }

    /// Creates an UndefinedFunctionError with a reason.
    #[must_use]
    pub fn with_reason(
        module: impl Into<String>,
        function: impl Into<String>,
        arity: u8,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            module: module.into(),
            function: function.into(),
            arity,
            reason: Some(reason.into()),
        }
    }

    /// Parses an OwnedTerm as an UndefinedFunctionError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.UndefinedFunctionError") {
            return None;
        }
        let map = term.as_map()?;
        let module_atom = map.get(&OwnedTerm::Atom(Atom::new("module")))?;
        let module = module_atom
            .atom_name()?
            .strip_prefix("Elixir.")
            .unwrap_or(module_atom.atom_name()?)
            .to_string();
        let function = map
            .get(&OwnedTerm::Atom(Atom::new("function")))?
            .atom_name()?
            .to_string();
        let arity = map
            .get(&OwnedTerm::Atom(Atom::new("arity")))?
            .as_integer()? as u8;
        let reason = map
            .get(&OwnedTerm::Atom(Atom::new("reason")))
            .and_then(|v| v.as_erlang_string());
        Some(Self {
            module,
            function,
            arity,
            reason,
        })
    }
}

impl ElixirExceptionExt for UndefinedFunctionError {
    fn module_name() -> &'static str {
        "Elixir.UndefinedFunctionError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());

        let module_atom = if self.module.starts_with("Elixir.") {
            self.module.clone()
        } else {
            format!("Elixir.{}", self.module)
        };

        map.insert(
            OwnedTerm::Atom(Atom::new("module")),
            OwnedTerm::Atom(Atom::new(&module_atom)),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("function")),
            OwnedTerm::Atom(Atom::new(&self.function)),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("arity")),
            OwnedTerm::Integer(self.arity as i64),
        );
        map.insert(
            OwnedTerm::Atom(Atom::new("reason")),
            self.reason.as_ref().map_or(OwnedTerm::elixir_nil(), |r| {
                OwnedTerm::Binary(r.clone().into_bytes())
            }),
        );
        OwnedTerm::Map(map)
    }
}

impl From<UndefinedFunctionError> for OwnedTerm {
    fn from(err: UndefinedFunctionError) -> Self {
        err.to_term()
    }
}

/// Elixir ArithmeticError exception.
///
/// Raised on arithmetic errors like division by zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArithmeticError {
    pub message: String,
}

impl ArithmeticError {
    /// Creates a new ArithmeticError.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Creates a "bad argument in arithmetic expression" error.
    #[must_use]
    pub fn bad_argument() -> Self {
        Self::new("bad argument in arithmetic expression")
    }

    /// Parses an OwnedTerm as an ArithmeticError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.ArithmeticError") {
            return None;
        }
        let map = term.as_map()?;
        let message = map
            .get(&OwnedTerm::Atom(Atom::new("message")))?
            .as_erlang_string()?;
        Some(Self { message })
    }
}

impl ElixirExceptionExt for ArithmeticError {
    fn module_name() -> &'static str {
        "Elixir.ArithmeticError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(
            OwnedTerm::Atom(Atom::new("message")),
            OwnedTerm::Binary(self.message.clone().into_bytes()),
        );
        OwnedTerm::Map(map)
    }
}

impl From<ArithmeticError> for OwnedTerm {
    fn from(err: ArithmeticError) -> Self {
        err.to_term()
    }
}

/// Elixir BadMapError exception.
///
/// Raised when a map operation is attempted on a non-map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadMapError {
    pub term: OwnedTerm,
}

impl BadMapError {
    /// Creates a new BadMapError.
    #[must_use]
    pub fn new(term: OwnedTerm) -> Self {
        Self { term }
    }

    /// Parses an OwnedTerm as a BadMapError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.BadMapError") {
            return None;
        }
        let map = term.as_map()?;
        let value = map.get(&OwnedTerm::Atom(Atom::new("term")))?.clone();
        Some(Self { term: value })
    }
}

impl ElixirExceptionExt for BadMapError {
    fn module_name() -> &'static str {
        "Elixir.BadMapError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(OwnedTerm::Atom(Atom::new("term")), self.term.clone());
        OwnedTerm::Map(map)
    }
}

impl From<BadMapError> for OwnedTerm {
    fn from(err: BadMapError) -> Self {
        err.to_term()
    }
}

/// Elixir BadFunctionError exception.
///
/// Raised when something is called as a function but is not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadFunctionError {
    pub term: OwnedTerm,
}

impl BadFunctionError {
    /// Creates a new BadFunctionError.
    #[must_use]
    pub fn new(term: OwnedTerm) -> Self {
        Self { term }
    }

    /// Parses an OwnedTerm as a BadFunctionError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.BadFunctionError") {
            return None;
        }
        let map = term.as_map()?;
        let value = map.get(&OwnedTerm::Atom(Atom::new("term")))?.clone();
        Some(Self { term: value })
    }
}

impl ElixirExceptionExt for BadFunctionError {
    fn module_name() -> &'static str {
        "Elixir.BadFunctionError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(OwnedTerm::Atom(Atom::new("term")), self.term.clone());
        OwnedTerm::Map(map)
    }
}

impl From<BadFunctionError> for OwnedTerm {
    fn from(err: BadFunctionError) -> Self {
        err.to_term()
    }
}

/// Elixir FunctionClauseError exception.
///
/// Raised when no function clause matches the given arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionClauseError {
    pub module: Option<String>,
    pub function: Option<String>,
    pub arity: Option<u8>,
    pub args: Option<OwnedTerm>,
}

impl FunctionClauseError {
    /// Creates a new FunctionClauseError.
    #[must_use]
    pub fn new(
        module: impl Into<String>,
        function: impl Into<String>,
        arity: u8,
        args: OwnedTerm,
    ) -> Self {
        Self {
            module: Some(module.into()),
            function: Some(function.into()),
            arity: Some(arity),
            args: Some(args),
        }
    }

    /// Creates an empty FunctionClauseError.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            module: None,
            function: None,
            arity: None,
            args: None,
        }
    }

    /// Parses an OwnedTerm as a FunctionClauseError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.FunctionClauseError") {
            return None;
        }
        let map = term.as_map()?;

        let module = map
            .get(&OwnedTerm::Atom(Atom::new("module")))
            .and_then(|m| m.atom_name())
            .map(|s| s.strip_prefix("Elixir.").unwrap_or(s).to_string());

        let function = map
            .get(&OwnedTerm::Atom(Atom::new("function")))
            .and_then(|f| f.atom_name())
            .map(|s| s.to_string());

        let arity = map
            .get(&OwnedTerm::Atom(Atom::new("arity")))
            .and_then(|a| a.as_integer())
            .map(|a| a as u8);

        let args = map
            .get(&OwnedTerm::Atom(Atom::new("args")))
            .filter(|a| !a.is_nil_atom())
            .cloned();

        Some(Self {
            module,
            function,
            arity,
            args,
        })
    }
}

impl ElixirExceptionExt for FunctionClauseError {
    fn module_name() -> &'static str {
        "Elixir.FunctionClauseError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());

        if let Some(ref module) = self.module {
            let module_atom = if module.starts_with("Elixir.") {
                module.clone()
            } else {
                format!("Elixir.{module}")
            };
            map.insert(
                OwnedTerm::Atom(Atom::new("module")),
                OwnedTerm::Atom(Atom::new(&module_atom)),
            );
        } else {
            map.insert(
                OwnedTerm::Atom(Atom::new("module")),
                OwnedTerm::elixir_nil(),
            );
        }

        if let Some(ref function) = self.function {
            map.insert(
                OwnedTerm::Atom(Atom::new("function")),
                OwnedTerm::Atom(Atom::new(function)),
            );
        } else {
            map.insert(
                OwnedTerm::Atom(Atom::new("function")),
                OwnedTerm::elixir_nil(),
            );
        }

        map.insert(
            OwnedTerm::Atom(Atom::new("arity")),
            self.arity
                .map_or(OwnedTerm::elixir_nil(), |a| OwnedTerm::Integer(a as i64)),
        );

        map.insert(
            OwnedTerm::Atom(Atom::new("args")),
            self.args.clone().unwrap_or_else(OwnedTerm::elixir_nil),
        );

        OwnedTerm::Map(map)
    }
}

impl From<FunctionClauseError> for OwnedTerm {
    fn from(err: FunctionClauseError) -> Self {
        err.to_term()
    }
}

/// Elixir CaseClauseError exception.
///
/// Raised when no case clause matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseClauseError {
    pub term: OwnedTerm,
}

impl CaseClauseError {
    /// Creates a new CaseClauseError.
    #[must_use]
    pub fn new(term: OwnedTerm) -> Self {
        Self { term }
    }

    /// Parses an OwnedTerm as a CaseClauseError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.CaseClauseError") {
            return None;
        }
        let map = term.as_map()?;
        let value = map.get(&OwnedTerm::Atom(Atom::new("term")))?.clone();
        Some(Self { term: value })
    }
}

impl ElixirExceptionExt for CaseClauseError {
    fn module_name() -> &'static str {
        "Elixir.CaseClauseError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(OwnedTerm::Atom(Atom::new("term")), self.term.clone());
        OwnedTerm::Map(map)
    }
}

impl From<CaseClauseError> for OwnedTerm {
    fn from(err: CaseClauseError) -> Self {
        err.to_term()
    }
}

/// Elixir CondClauseError exception.
///
/// Raised when no cond clause evaluates to a truthy value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CondClauseError;

impl CondClauseError {
    /// Creates a new CondClauseError.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Parses an OwnedTerm as a CondClauseError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.CondClauseError") {
            return None;
        }
        Some(Self)
    }
}

impl ElixirExceptionExt for CondClauseError {
    fn module_name() -> &'static str {
        "Elixir.CondClauseError"
    }

    fn to_term(&self) -> OwnedTerm {
        OwnedTerm::Map(exception_base(Self::module_name()))
    }
}

impl From<CondClauseError> for OwnedTerm {
    fn from(err: CondClauseError) -> Self {
        err.to_term()
    }
}

/// Elixir WithClauseError exception.
///
/// Raised when no with clause matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithClauseError {
    pub term: OwnedTerm,
}

impl WithClauseError {
    /// Creates a new WithClauseError.
    #[must_use]
    pub fn new(term: OwnedTerm) -> Self {
        Self { term }
    }

    /// Parses an OwnedTerm as a WithClauseError.
    #[must_use]
    pub fn from_term(term: &OwnedTerm) -> Option<Self> {
        if term.elixir_struct_module() != Some("Elixir.WithClauseError") {
            return None;
        }
        let map = term.as_map()?;
        let value = map.get(&OwnedTerm::Atom(Atom::new("term")))?.clone();
        Some(Self { term: value })
    }
}

impl ElixirExceptionExt for WithClauseError {
    fn module_name() -> &'static str {
        "Elixir.WithClauseError"
    }

    fn to_term(&self) -> OwnedTerm {
        let mut map = exception_base(Self::module_name());
        map.insert(OwnedTerm::Atom(Atom::new("term")), self.term.clone());
        OwnedTerm::Map(map)
    }
}

impl From<WithClauseError> for OwnedTerm {
    fn from(err: WithClauseError) -> Self {
        err.to_term()
    }
}
