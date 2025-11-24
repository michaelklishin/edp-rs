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

use crate::errors::DecodeError;
use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, LazyLock};

const COMMON_ATOMS: [(&str, usize); 14] = [
    ("ok", 0),
    ("error", 1),
    ("true", 2),
    ("false", 3),
    ("nil", 4),
    ("undefined", 5),
    ("normal", 6),
    ("shutdown", 7),
    ("infinity", 8),
    ("badarg", 9),
    ("badarith", 10),
    ("badmatch", 11),
    ("noproc", 12),
    ("timeout", 13),
];

static CACHED_ATOMS: [LazyLock<Arc<str>>; 14] = [
    LazyLock::new(|| Arc::from("ok")),
    LazyLock::new(|| Arc::from("error")),
    LazyLock::new(|| Arc::from("true")),
    LazyLock::new(|| Arc::from("false")),
    LazyLock::new(|| Arc::from("nil")),
    LazyLock::new(|| Arc::from("undefined")),
    LazyLock::new(|| Arc::from("normal")),
    LazyLock::new(|| Arc::from("shutdown")),
    LazyLock::new(|| Arc::from("infinity")),
    LazyLock::new(|| Arc::from("badarg")),
    LazyLock::new(|| Arc::from("badarith")),
    LazyLock::new(|| Arc::from("badmatch")),
    LazyLock::new(|| Arc::from("noproc")),
    LazyLock::new(|| Arc::from("timeout")),
];

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Atom {
    pub name: Arc<str>,
}

impl Atom {
    pub const OK: &'static str = "ok";
    pub const ERROR: &'static str = "error";
    pub const TRUE: &'static str = "true";
    pub const FALSE: &'static str = "false";
    pub const NIL: &'static str = "nil";
    pub const UNDEFINED: &'static str = "undefined";
    pub const NORMAL: &'static str = "normal";
    pub const SHUTDOWN: &'static str = "shutdown";

    pub fn new<S: AsRef<str>>(name: S) -> Self {
        let name_ref = name.as_ref();

        for (atom_str, idx) in &COMMON_ATOMS {
            if *atom_str == name_ref {
                return Atom {
                    name: CACHED_ATOMS[*idx].clone(),
                };
            }
        }

        Atom {
            name: Arc::from(name_ref),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.name.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.as_str() == Self::OK
    }

    #[inline]
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.as_str() == Self::ERROR
    }

    #[inline]
    #[must_use]
    pub fn is_true(&self) -> bool {
        self.as_str() == Self::TRUE
    }

    #[inline]
    #[must_use]
    pub fn is_false(&self) -> bool {
        self.as_str() == Self::FALSE
    }

    #[inline]
    #[must_use]
    pub fn is_nil(&self) -> bool {
        self.as_str() == Self::NIL
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<String> for Atom {
    fn from(s: String) -> Self {
        Atom::new(s)
    }
}

impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Atom::new(s)
    }
}

impl AsRef<str> for Atom {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl Deref for Atom {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl Borrow<str> for Atom {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq<str> for Atom {
    fn eq(&self, other: &str) -> bool {
        &*self.name == other
    }
}

impl PartialEq<&str> for Atom {
    fn eq(&self, other: &&str) -> bool {
        &*self.name == *other
    }
}

impl PartialEq<Atom> for str {
    fn eq(&self, other: &Atom) -> bool {
        self == &*other.name
    }
}

impl PartialEq<Atom> for &str {
    fn eq(&self, other: &Atom) -> bool {
        *self == &*other.name
    }
}

impl PartialEq<Arc<str>> for Atom {
    fn eq(&self, other: &Arc<str>) -> bool {
        &self.name == other
    }
}

impl PartialEq<Atom> for Arc<str> {
    fn eq(&self, other: &Atom) -> bool {
        self == &other.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Positive,
    Negative,
}

impl Sign {
    #[inline]
    pub fn is_negative(self) -> bool {
        matches!(self, Sign::Negative)
    }

    #[inline]
    pub fn is_positive(self) -> bool {
        matches!(self, Sign::Positive)
    }
}

impl From<bool> for Sign {
    fn from(b: bool) -> Self {
        if b { Sign::Negative } else { Sign::Positive }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigInt {
    pub sign: Sign,
    pub digits: Vec<u8>,
}

impl BigInt {
    #[inline]
    pub fn new<S: Into<Sign>>(sign: S, digits: Vec<u8>) -> Self {
        BigInt {
            sign: sign.into(),
            digits,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalPid {
    pub node: Atom,
    pub id: u32,
    pub serial: u32,
    pub creation: u32,
}

impl ExternalPid {
    #[inline]
    pub fn new(node: Atom, id: u32, serial: u32, creation: u32) -> Self {
        ExternalPid {
            node,
            id,
            serial,
            creation,
        }
    }

    pub fn from_string(node: Atom, pid_str: &str) -> Result<Self, DecodeError> {
        let trimmed = pid_str.trim();

        if !trimmed.starts_with('<') || !trimmed.ends_with('>') {
            return Err(DecodeError::InvalidPidFormat(format!(
                "PID string must be in format <id.serial.creation>, got: {}",
                pid_str
            )));
        }

        let inner = &trimmed[1..trimmed.len() - 1];
        let parts: Vec<&str> = inner.split('.').collect();

        if parts.len() != 3 {
            return Err(DecodeError::InvalidPidFormat(format!(
                "PID string must have exactly 3 parts separated by dots, got: {}",
                pid_str
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            DecodeError::InvalidPidFormat(format!("Invalid id in PID string: {}", parts[0]))
        })?;
        let serial = parts[1].parse::<u32>().map_err(|_| {
            DecodeError::InvalidPidFormat(format!("Invalid serial in PID string: {}", parts[1]))
        })?;
        let creation = parts[2].parse::<u32>().map_err(|_| {
            DecodeError::InvalidPidFormat(format!("Invalid creation in PID string: {}", parts[2]))
        })?;

        Ok(ExternalPid::new(node, id, serial, creation))
    }
}

impl fmt::Display for ExternalPid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}.{}.{}>", self.id, self.serial, self.creation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalPort {
    pub node: Atom,
    pub id: u64,
    pub creation: u32,
}

impl ExternalPort {
    #[inline]
    pub fn new(node: Atom, id: u64, creation: u32) -> Self {
        ExternalPort { node, id, creation }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalReference {
    pub node: Atom,
    pub creation: u32,
    pub ids: Vec<u32>,
}

impl ExternalReference {
    #[inline]
    pub fn new(node: Atom, creation: u32, ids: Vec<u32>) -> Self {
        ExternalReference {
            node,
            creation,
            ids,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExternalFun {
    pub module: Atom,
    pub function: Atom,
    pub arity: u8,
}

impl ExternalFun {
    #[inline]
    pub fn new(module: Atom, function: Atom, arity: u8) -> Self {
        ExternalFun {
            module,
            function,
            arity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InternalFun {
    pub arity: u8,
    pub uniq: [u8; 16],
    pub index: u32,
    pub num_free: u32,
    pub module: Atom,
    pub old_index: u32,
    pub old_uniq: u32,
    pub pid: ExternalPid,
    pub free_vars: Vec<crate::term::OwnedTerm>,
}

impl InternalFun {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        arity: u8,
        uniq: [u8; 16],
        index: u32,
        num_free: u32,
        module: Atom,
        old_index: u32,
        old_uniq: u32,
        pid: ExternalPid,
        free_vars: Vec<crate::term::OwnedTerm>,
    ) -> Self {
        InternalFun {
            arity,
            uniq,
            index,
            num_free,
            module,
            old_index,
            old_uniq,
            pid,
            free_vars,
        }
    }
}
