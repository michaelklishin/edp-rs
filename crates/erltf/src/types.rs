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
use crate::term::OwnedTerm;
use bytes::Bytes;
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

/// Represents an Erlang PID originating from a remote node.
/// These are easy to get wrong when encoding and decoding
/// due to the special LOCAL_EXT encoding.
///
/// So when a PID is received via LOCAL_EXT encoding (tag 121), the original
/// raw bytes are preserved in `local_ext_bytes` field to allow for transparent
/// re-encoding when sending the PID back to the remote node.
///
/// N.B.: the `local_ext_bytes` field is excluded from equality, hash, and ordering
/// comparisons as it nothing but an implementation detail. Two PIDs are equal if their (node, id, serial, creation) match,
/// regardless of how they were encoded.
#[derive(Debug, Clone)]
pub struct ExternalPid {
    pub node: Atom,
    pub id: u32,
    pub serial: u32,
    pub creation: u32,
    /// If this PID was decoded from LOCAL_EXT, this contains the hash (8 bytes)
    /// and nested term bytes for transparent re-encoding. The LOCAL_EXT tag
    /// is added during encoding.
    ///
    /// `Bytes` offer zero-copy cloning.
    pub local_ext_bytes: Option<Bytes>,
}

impl PartialEq for ExternalPid {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
            && self.id == other.id
            && self.serial == other.serial
            && self.creation == other.creation
    }
}

impl Eq for ExternalPid {}

impl std::hash::Hash for ExternalPid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state);
        self.id.hash(state);
        self.serial.hash(state);
        self.creation.hash(state);
    }
}

impl PartialOrd for ExternalPid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExternalPid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.node, self.id, self.serial, self.creation).cmp(&(
            &other.node,
            other.id,
            other.serial,
            other.creation,
        ))
    }
}

impl ExternalPid {
    #[inline]
    pub fn new(node: Atom, id: u32, serial: u32, creation: u32) -> Self {
        ExternalPid {
            node,
            id,
            serial,
            creation,
            local_ext_bytes: None,
        }
    }

    /// Creates a new PID with preserved LOCAL_EXT bytes.
    ///
    /// Accepts any type that can be converted to `Bytes` (e.g., `Vec<u8>`, `&[u8]`, `Bytes`).
    #[inline]
    pub fn with_local_ext_bytes(
        node: Atom,
        id: u32,
        serial: u32,
        creation: u32,
        local_ext_bytes: impl Into<Bytes>,
    ) -> Self {
        ExternalPid {
            node,
            id,
            serial,
            creation,
            local_ext_bytes: Some(local_ext_bytes.into()),
        }
    }

    /// Returns true if this PID's construction originates from a LOCAL_EXT encoding operation.
    #[inline]
    #[must_use]
    pub fn is_local_ext(&self) -> bool {
        self.local_ext_bytes.is_some()
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

    #[inline]
    #[must_use]
    pub fn to_erl_pid_string(&self) -> String {
        format!("<0.{}.{}>", self.id, self.serial)
    }

    /// Parses a PID string in the format used by `erlang:pid_to_list/1`: `<0.{id}.{serial}>`.
    /// This is the format returned by `to_erl_pid_string`.
    ///
    /// The node and creation must be passed in because they cannot be inferred from the input.
    pub fn from_erl_pid_string(
        node: Atom,
        pid_str: &str,
        creation: u32,
    ) -> Result<Self, DecodeError> {
        let trimmed = pid_str.trim();

        if !trimmed.starts_with('<') || !trimmed.ends_with('>') {
            return Err(DecodeError::InvalidPidFormat(format!(
                "PID string must be in format <0.id.serial>, got: {}",
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

        // First part should be 0 (indicating local node in Erlang's pid_to_list format)
        if parts[0] != "0" {
            return Err(DecodeError::InvalidPidFormat(format!(
                "PID string must start with <0., got: {}",
                pid_str
            )));
        }

        let id = parts[1].parse::<u32>().map_err(|_| {
            DecodeError::InvalidPidFormat(format!("Invalid id in PID string: {}", parts[1]))
        })?;
        let serial = parts[2].parse::<u32>().map_err(|_| {
            DecodeError::InvalidPidFormat(format!("Invalid serial in PID string: {}", parts[2]))
        })?;

        Ok(ExternalPid::new(node, id, serial, creation))
    }

    #[inline]
    #[must_use]
    pub fn to_charlist_term(&self) -> OwnedTerm {
        OwnedTerm::charlist(self.to_erl_pid_string())
    }
}

impl fmt::Display for ExternalPid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}.{}.{}>", self.id, self.serial, self.creation)
    }
}

/// Represents an Erlang port originating from a remote node.
///
/// Like PIDs, ports can be encoded via LOCAL_EXT. The `local_ext_bytes` field
/// preserves the original encoding for transparent re-encoding.
///
/// Note: The `local_ext_bytes` field is excluded from equality, hash, and ordering
/// comparisons.
#[derive(Debug, Clone)]
pub struct ExternalPort {
    pub node: Atom,
    pub id: u64,
    pub creation: u32,
    /// If this port was decoded from LOCAL_EXT, this contains the hash (8 bytes)
    /// and nested term bytes for transparent re-encoding.
    ///
    /// Uses `Bytes` for efficient zero-copy cloning of frequently-decoded terms.
    pub local_ext_bytes: Option<Bytes>,
}

impl PartialEq for ExternalPort {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.id == other.id && self.creation == other.creation
    }
}

impl Eq for ExternalPort {}

impl std::hash::Hash for ExternalPort {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state);
        self.id.hash(state);
        self.creation.hash(state);
    }
}

impl PartialOrd for ExternalPort {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExternalPort {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.node, self.id, self.creation).cmp(&(&other.node, other.id, other.creation))
    }
}

impl ExternalPort {
    #[inline]
    pub fn new(node: Atom, id: u64, creation: u32) -> Self {
        ExternalPort {
            node,
            id,
            creation,
            local_ext_bytes: None,
        }
    }

    /// Creates a new port with preserved LOCAL_EXT bytes.
    ///
    /// Accepts any type that can be converted to `Bytes` (e.g., `Vec<u8>`, `&[u8]`, `Bytes`).
    #[inline]
    pub fn with_local_ext_bytes(
        node: Atom,
        id: u64,
        creation: u32,
        local_ext_bytes: impl Into<Bytes>,
    ) -> Self {
        ExternalPort {
            node,
            id,
            creation,
            local_ext_bytes: Some(local_ext_bytes.into()),
        }
    }

    /// Returns true if this port was decoded from LOCAL_EXT encoding.
    #[inline]
    #[must_use]
    pub fn is_local_ext(&self) -> bool {
        self.local_ext_bytes.is_some()
    }
}

/// Represents an Erlang reference originating from a remote node.
///
/// Like PIDs, references can be encoded via LOCAL_EXT. The `local_ext_bytes` field
/// preserves the original encoding for transparent re-encoding.
///
/// Note: The `local_ext_bytes` field is excluded from equality, hash, and ordering
/// comparisons.
#[derive(Debug, Clone)]
pub struct ExternalReference {
    pub node: Atom,
    pub creation: u32,
    pub ids: Vec<u32>,
    /// If this reference was decoded from LOCAL_EXT, this contains the hash (8 bytes)
    /// and nested term bytes for transparent re-encoding.
    ///
    /// `Bytes` offer zero-copy cloning.
    pub local_ext_bytes: Option<Bytes>,
}

impl PartialEq for ExternalReference {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.creation == other.creation && self.ids == other.ids
    }
}

impl Eq for ExternalReference {}

impl std::hash::Hash for ExternalReference {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state);
        self.creation.hash(state);
        self.ids.hash(state);
    }
}

impl PartialOrd for ExternalReference {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExternalReference {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.node, self.creation, &self.ids).cmp(&(&other.node, other.creation, &other.ids))
    }
}

impl ExternalReference {
    #[inline]
    pub fn new(node: Atom, creation: u32, ids: Vec<u32>) -> Self {
        ExternalReference {
            node,
            creation,
            ids,
            local_ext_bytes: None,
        }
    }

    /// Creates a new reference with preserved LOCAL_EXT bytes.
    ///
    /// Accepts any type that can be converted to `Bytes` (e.g., `Vec<u8>`, `&[u8]`, `Bytes`).
    #[inline]
    pub fn with_local_ext_bytes(
        node: Atom,
        creation: u32,
        ids: Vec<u32>,
        local_ext_bytes: impl Into<Bytes>,
    ) -> Self {
        ExternalReference {
            node,
            creation,
            ids,
            local_ext_bytes: Some(local_ext_bytes.into()),
        }
    }

    /// Returns true if this reference was decoded from LOCAL_EXT encoding.
    #[inline]
    #[must_use]
    pub fn is_local_ext(&self) -> bool {
        self.local_ext_bytes.is_some()
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mfa {
    pub module: Atom,
    pub function: Atom,
    pub arity: u8,
}

impl Mfa {
    #[inline]
    pub fn new<M, F>(module: M, function: F, arity: u8) -> Self
    where
        M: Into<Atom>,
        F: Into<Atom>,
    {
        Mfa {
            module: module.into(),
            function: function.into(),
            arity,
        }
    }

    pub fn try_from_term(term: &OwnedTerm) -> Option<Self> {
        match term {
            OwnedTerm::Tuple(elems) if elems.len() == 3 => {
                let module = elems[0].as_atom()?.clone();
                let function = elems[1].as_atom()?.clone();
                let arity = match &elems[2] {
                    OwnedTerm::Integer(n) if *n >= 0 && *n <= 255 => *n as u8,
                    _ => return None,
                };
                Some(Mfa {
                    module,
                    function,
                    arity,
                })
            }
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn to_term(&self) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(self.module.clone()),
            OwnedTerm::Atom(self.function.clone()),
            OwnedTerm::Integer(self.arity as i64),
        ])
    }
}

impl fmt::Display for Mfa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}/{}", self.module, self.function, self.arity)
    }
}

impl From<ExternalFun> for Mfa {
    fn from(fun: ExternalFun) -> Self {
        Mfa {
            module: fun.module,
            function: fun.function,
            arity: fun.arity,
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
    pub free_vars: Vec<OwnedTerm>,
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
        free_vars: Vec<OwnedTerm>,
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
