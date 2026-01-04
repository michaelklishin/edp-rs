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

use crate::errors::TermConversionError;
use crate::types::{
    Atom, BigInt, ExternalFun, ExternalPid, ExternalPort, ExternalReference, InternalFun, Mfa, Sign,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::mem::discriminant;
use std::ops::Index;
use std::sync::{Arc, OnceLock};

#[cfg(feature = "serde")]
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum OwnedTerm {
    Atom(Atom),
    Integer(i64),
    Float(f64),
    Pid(ExternalPid),
    Port(ExternalPort),
    Reference(ExternalReference),
    Binary(Vec<u8>),
    BitBinary {
        bytes: Vec<u8>,
        bits: u8,
    },
    String(String),
    List(Vec<Self>),
    ImproperList {
        elements: Vec<Self>,
        tail: Box<OwnedTerm>,
    },
    Map(BTreeMap<Self, Self>),
    Tuple(Vec<Self>),
    BigInt(BigInt),
    ExternalFun(ExternalFun),
    InternalFun(Box<InternalFun>),
    #[default]
    Nil,
}

/// Provides unified key-value access over both Erlang maps and proplists.
///
/// # Example
/// ```
/// use erltf::{OwnedTerm, KeyValueAccess};
/// use std::collections::BTreeMap;
///
/// // Works with maps
/// let map = OwnedTerm::map(BTreeMap::from([
///     (OwnedTerm::atom("name"), OwnedTerm::atom("value"))
/// ]));
/// assert_eq!(map.kv_get_atom_string("name"), Some("value".to_string()));
///
/// // Works with proplists
/// let proplist = OwnedTerm::list(vec![
///     OwnedTerm::Tuple(vec![OwnedTerm::atom("name"), OwnedTerm::atom("value")])
/// ]);
/// assert_eq!(proplist.kv_get_atom_string("name"), Some("value".to_string()));
/// ```
pub trait KeyValueAccess {
    /// Retrieve the value associated with an atom key, returning the raw term
    fn kv_get(&self, key: &str) -> Option<&OwnedTerm>;

    /// Retrieve an i64 value by key
    fn kv_get_i64(&self, key: &str) -> Option<i64> {
        self.kv_get(key).and_then(|t| t.as_integer())
    }

    /// Retrieve an i64 value by key or return the default if the key is missing
    fn kv_get_i64_or(&self, key: &str, default: i64) -> i64 {
        self.kv_get_i64(key).unwrap_or(default)
    }

    /// Retrieve a boolean value by key
    fn kv_get_bool(&self, key: &str) -> Option<bool> {
        self.kv_get(key).and_then(|t| t.as_bool())
    }

    /// Retrieve a boolean value by key, or return the default if the key is missing
    fn kv_get_bool_or(&self, key: &str, default: bool) -> bool {
        self.kv_get_bool(key).unwrap_or(default)
    }

    /// Retrieve an atom value by key
    fn kv_get_atom(&self, key: &str) -> Option<&Atom> {
        self.kv_get(key).and_then(|t| t.as_atom())
    }

    /// Retrieve an atom value as a string by key
    fn kv_get_atom_string(&self, key: &str) -> Option<String> {
        self.kv_get_atom(key).map(|a| a.to_string())
    }

    /// Retrieve an atom value as a string by key or return the default if the key is missing
    fn kv_get_atom_string_or(&self, key: &str, default: &str) -> String {
        self.kv_get_atom_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    /// Retrieve a string value by key
    fn kv_get_string(&self, key: &str) -> Option<String> {
        self.kv_get(key).and_then(|t| t.as_erlang_string())
    }

    /// Retrieve a string value by key or return the default if the key is missing
    fn kv_get_string_or(&self, key: &str, default: &str) -> String {
        self.kv_get_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    /// Retrieve a PID value by key
    fn kv_get_pid(&self, key: &str) -> Option<&ExternalPid> {
        self.kv_get(key).and_then(|t| t.as_pid())
    }

    /// Retrieve a PID value as a formatted string by key
    fn kv_get_pid_string(&self, key: &str) -> Option<String> {
        self.kv_get_pid(key).map(|p| p.to_string())
    }

    /// Retrieve a PID value as a formatted string by key or return the default if the key is missing
    fn kv_get_pid_string_or(&self, key: &str, default: &str) -> String {
        self.kv_get_pid_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    /// Retrieve an MFA value as a formatted string by key
    fn kv_get_mfa_string(&self, key: &str) -> Option<String> {
        self.kv_get(key).and_then(|t| t.format_as_mfa())
    }

    /// Retrieve an MFA value as a formatted string by key or return the default if the key is missing
    fn kv_get_mfa_string_or(&self, key: &str, default: &str) -> String {
        self.kv_get_mfa_string(key)
            .unwrap_or_else(|| default.to_string())
    }
}

impl KeyValueAccess for OwnedTerm {
    fn kv_get(&self, key: &str) -> Option<&OwnedTerm> {
        match self {
            OwnedTerm::Map(_) => self.map_get_atom_key(key),
            OwnedTerm::List(_) => self.proplist_get_atom_key(key),
            _ => None,
        }
    }
}

impl OwnedTerm {
    pub fn atom<S: AsRef<str>>(name: S) -> Self {
        OwnedTerm::Atom(Atom::new(name))
    }

    pub fn integer(value: i64) -> Self {
        OwnedTerm::Integer(value)
    }

    pub fn float(value: f64) -> Self {
        OwnedTerm::Float(value)
    }

    pub fn binary(data: Vec<u8>) -> Self {
        OwnedTerm::Binary(data)
    }

    pub fn string<S: Into<String>>(value: S) -> Self {
        OwnedTerm::String(value.into())
    }

    pub fn list(elements: Vec<Self>) -> Self {
        OwnedTerm::List(elements)
    }

    pub fn improper_list(elements: Vec<Self>, tail: Self) -> Self {
        OwnedTerm::ImproperList {
            elements,
            tail: Box::new(tail),
        }
    }

    pub fn map(entries: BTreeMap<Self, Self>) -> Self {
        OwnedTerm::Map(entries)
    }

    pub fn tuple(elements: Vec<Self>) -> Self {
        OwnedTerm::Tuple(elements)
    }

    pub fn boolean(value: bool) -> Self {
        OwnedTerm::atom(if value { "true" } else { "false" })
    }

    pub fn ok() -> Self {
        OwnedTerm::atom("ok")
    }

    pub fn error() -> Self {
        OwnedTerm::atom("error")
    }

    pub fn ok_tuple(value: OwnedTerm) -> Self {
        OwnedTerm::Tuple(vec![OwnedTerm::ok(), value])
    }

    pub fn error_tuple(reason: OwnedTerm) -> Self {
        OwnedTerm::Tuple(vec![OwnedTerm::error(), reason])
    }

    pub fn nil() -> Self {
        OwnedTerm::Nil
    }

    #[inline]
    #[must_use]
    pub fn is_atom(&self) -> bool {
        matches!(self, OwnedTerm::Atom(_))
    }

    #[inline]
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, OwnedTerm::Integer(_))
    }

    #[inline]
    #[must_use]
    pub fn is_list(&self) -> bool {
        matches!(self, OwnedTerm::List(_) | OwnedTerm::Nil)
    }

    #[inline]
    #[must_use]
    pub fn is_map(&self) -> bool {
        matches!(self, OwnedTerm::Map(_))
    }

    #[inline]
    #[must_use]
    pub fn is_tuple(&self) -> bool {
        matches!(self, OwnedTerm::Tuple(_))
    }

    #[inline]
    #[must_use]
    pub fn as_atom(&self) -> Option<&Atom> {
        match self {
            OwnedTerm::Atom(a) => Some(a),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            OwnedTerm::Integer(i) => Some(*i),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            OwnedTerm::Float(f) => Some(*f),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            OwnedTerm::Binary(b) => Some(b),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            OwnedTerm::String(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_list(&self) -> Option<&[OwnedTerm]> {
        match self {
            OwnedTerm::List(l) => Some(l),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_map(&self) -> Option<&BTreeMap<Self, Self>> {
        match self {
            OwnedTerm::Map(m) => Some(m),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_tuple(&self) -> Option<&[OwnedTerm]> {
        match self {
            OwnedTerm::Tuple(t) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            OwnedTerm::List(l) => Some(l),
            _ => None,
        }
    }

    #[inline]
    pub fn as_map_mut(&mut self) -> Option<&mut BTreeMap<Self, Self>> {
        match self {
            OwnedTerm::Map(m) => Some(m),
            _ => None,
        }
    }

    #[inline]
    pub fn as_tuple_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            OwnedTerm::Tuple(t) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn as_binary_mut(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            OwnedTerm::Binary(b) => Some(b),
            _ => None,
        }
    }

    #[inline]
    pub fn try_as_integer(&self) -> Result<i64, TermConversionError> {
        self.as_integer().ok_or(TermConversionError::WrongType {
            expected: "Integer",
            actual: self.type_name(),
        })
    }

    #[inline]
    pub fn try_as_float(&self) -> Result<f64, TermConversionError> {
        self.as_float().ok_or(TermConversionError::WrongType {
            expected: "Float",
            actual: self.type_name(),
        })
    }

    #[inline]
    pub fn try_as_atom(&self) -> Result<&Atom, TermConversionError> {
        self.as_atom().ok_or(TermConversionError::WrongType {
            expected: "Atom",
            actual: self.type_name(),
        })
    }

    #[inline]
    pub fn try_as_binary(&self) -> Result<&[u8], TermConversionError> {
        self.as_binary().ok_or(TermConversionError::WrongType {
            expected: "Binary",
            actual: self.type_name(),
        })
    }

    #[inline]
    pub fn try_as_string(&self) -> Result<&str, TermConversionError> {
        self.as_string().ok_or(TermConversionError::WrongType {
            expected: "String",
            actual: self.type_name(),
        })
    }

    #[inline]
    pub fn try_as_list(&self) -> Result<&[OwnedTerm], TermConversionError> {
        self.as_list().ok_or(TermConversionError::WrongType {
            expected: "List",
            actual: self.type_name(),
        })
    }

    #[inline]
    pub fn try_as_tuple(&self) -> Result<&[OwnedTerm], TermConversionError> {
        self.as_tuple().ok_or(TermConversionError::WrongType {
            expected: "Tuple",
            actual: self.type_name(),
        })
    }

    #[inline]
    pub fn try_as_map(&self) -> Result<&BTreeMap<Self, Self>, TermConversionError> {
        self.as_map().ok_or(TermConversionError::WrongType {
            expected: "Map",
            actual: self.type_name(),
        })
    }

    pub fn into_map_iter(
        self,
    ) -> Result<impl Iterator<Item = (OwnedTerm, OwnedTerm)>, TermConversionError> {
        match self {
            OwnedTerm::Map(m) => Ok(m.into_iter()),
            _ => Err(TermConversionError::WrongType {
                expected: "Map",
                actual: self.type_name(),
            }),
        }
    }

    #[inline]
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            OwnedTerm::Atom(_) => "Atom",
            OwnedTerm::Integer(_) => "Integer",
            OwnedTerm::Float(_) => "Float",
            OwnedTerm::Pid(_) => "Pid",
            OwnedTerm::Port(_) => "Port",
            OwnedTerm::Reference(_) => "Reference",
            OwnedTerm::Binary(_) => "Binary",
            OwnedTerm::BitBinary { .. } => "BitBinary",
            OwnedTerm::String(_) => "String",
            OwnedTerm::List(_) => "List",
            OwnedTerm::ImproperList { .. } => "ImproperList",
            OwnedTerm::Map(_) => "Map",
            OwnedTerm::Tuple(_) => "Tuple",
            OwnedTerm::BigInt(_) => "BigInt",
            OwnedTerm::ExternalFun(_) => "ExternalFun",
            OwnedTerm::InternalFun(_) => "InternalFun",
            OwnedTerm::Nil => "Nil",
        }
    }

    #[inline]
    #[must_use]
    pub fn atom_name(&self) -> Option<&str> {
        match self {
            OwnedTerm::Atom(a) => Some(&a.name),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_atom_with_name(&self, name: &str) -> bool {
        match self {
            OwnedTerm::Atom(a) => a == name,
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_true(&self) -> bool {
        self.as_bool() == Some(true)
    }

    #[inline]
    #[must_use]
    pub fn is_false(&self) -> bool {
        self.as_bool() == Some(false)
    }

    #[inline]
    #[must_use]
    pub fn is_undefined(&self) -> bool {
        self.is_atom_with_name("undefined")
    }

    #[inline]
    #[must_use]
    pub fn is_nil_atom(&self) -> bool {
        self.is_atom_with_name("nil")
    }

    #[inline]
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        self.atom_name().and_then(|name| match name {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
    }

    pub fn into_ok_value(self) -> Option<OwnedTerm> {
        match self {
            OwnedTerm::Tuple(mut elements) if elements.len() == 2 => {
                if elements[0] == OwnedTerm::ok() {
                    Some(elements.swap_remove(1))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn into_rex_response(self) -> Result<OwnedTerm, TermConversionError> {
        match self {
            OwnedTerm::Tuple(mut elements) if elements.len() == 2 => {
                if elements[0].is_atom_with_name("rex") {
                    Ok(elements.swap_remove(1))
                } else {
                    Err(TermConversionError::WrongType {
                        expected: "{rex, Result} tuple",
                        actual: "tuple with different first element",
                    })
                }
            }
            _ => Err(TermConversionError::WrongType {
                expected: "{rex, Result} tuple",
                actual: self.type_name(),
            }),
        }
    }

    pub fn into_error_reason(self) -> Option<OwnedTerm> {
        match self {
            OwnedTerm::Tuple(mut elements) if elements.len() == 2 => {
                if elements[0] == OwnedTerm::error() {
                    Some(elements.swap_remove(1))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn map_get(&self, key: &OwnedTerm) -> Option<&OwnedTerm> {
        match self {
            OwnedTerm::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn get<I: TermIndex>(&self, index: I) -> Option<&OwnedTerm> {
        index.get_from_term(self)
    }

    pub fn iter(&self) -> OwnedTermIter<'_> {
        match self {
            OwnedTerm::List(elements) | OwnedTerm::Tuple(elements) => {
                OwnedTermIter::Slice(elements.iter())
            }
            OwnedTerm::Nil => OwnedTermIter::Empty,
            _ => OwnedTermIter::Empty,
        }
    }

    pub fn proplist_get_atom_key(&self, key: &str) -> Option<&OwnedTerm> {
        match self {
            OwnedTerm::List(elements) => {
                for element in elements {
                    if let OwnedTerm::Tuple(tuple_elements) = element
                        && tuple_elements.len() == 2
                        && let OwnedTerm::Atom(atom) = &tuple_elements[0]
                        && atom == key
                    {
                        return Some(&tuple_elements[1]);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn is_proplist(&self) -> bool {
        match self {
            OwnedTerm::List(elements) => elements.iter().all(Self::is_proplist_element),
            OwnedTerm::Nil => true,
            _ => false,
        }
    }

    pub fn is_proplist_element(element: &OwnedTerm) -> bool {
        match element {
            OwnedTerm::Tuple(elements) if elements.len() == 2 => {
                matches!(
                    &elements[0],
                    OwnedTerm::Atom(_) | OwnedTerm::Binary(_) | OwnedTerm::String(_)
                )
            }
            OwnedTerm::Atom(_) => true,
            _ => false,
        }
    }

    pub fn normalize_proplist(&self) -> Result<OwnedTerm, TermConversionError> {
        match self {
            OwnedTerm::List(elements) => {
                let normalized: Vec<OwnedTerm> = elements
                    .iter()
                    .filter_map(|el| match el {
                        OwnedTerm::Tuple(t) if t.len() == 2 => Some(el.clone()),
                        OwnedTerm::Atom(a) => Some(OwnedTerm::Tuple(vec![
                            OwnedTerm::Atom(a.clone()),
                            OwnedTerm::boolean(true),
                        ])),
                        _ => None,
                    })
                    .collect();
                Ok(OwnedTerm::List(normalized))
            }
            OwnedTerm::Nil => Ok(OwnedTerm::List(vec![])),
            _ => Err(TermConversionError::WrongType {
                expected: "List",
                actual: self.type_name(),
            }),
        }
    }

    pub fn proplist_to_map(&self) -> Result<OwnedTerm, TermConversionError> {
        match self {
            OwnedTerm::List(elements) => {
                let mut map = BTreeMap::new();
                for element in elements {
                    match element {
                        OwnedTerm::Tuple(t) if t.len() == 2 => {
                            map.insert(t[0].clone(), t[1].clone());
                        }
                        OwnedTerm::Atom(a) => {
                            map.insert(OwnedTerm::Atom(a.clone()), OwnedTerm::boolean(true));
                        }
                        _ => {}
                    }
                }
                Ok(OwnedTerm::Map(map))
            }
            OwnedTerm::Map(_) => Ok(self.clone()),
            OwnedTerm::Nil => Ok(OwnedTerm::Map(BTreeMap::new())),
            _ => Err(TermConversionError::WrongType {
                expected: "List or Map",
                actual: self.type_name(),
            }),
        }
    }

    pub fn map_to_proplist(&self) -> Result<OwnedTerm, TermConversionError> {
        match self {
            OwnedTerm::Map(map) => {
                let elements: Vec<OwnedTerm> = map
                    .iter()
                    .map(|(k, v)| OwnedTerm::Tuple(vec![k.clone(), v.clone()]))
                    .collect();
                Ok(OwnedTerm::List(elements))
            }
            OwnedTerm::List(_) | OwnedTerm::Nil => Ok(self.clone()),
            _ => Err(TermConversionError::WrongType {
                expected: "Map or List",
                actual: self.type_name(),
            }),
        }
    }

    pub fn to_map_recursive(&self) -> Result<OwnedTerm, TermConversionError> {
        match self {
            OwnedTerm::List(elements) if elements.is_empty() => Ok(OwnedTerm::List(vec![])),
            OwnedTerm::List(_) if self.is_proplist() => {
                let normalized = self.normalize_proplist()?;
                let map = normalized.proplist_to_map()?;
                if let OwnedTerm::Map(m) = map {
                    let mut result = BTreeMap::new();
                    for (k, v) in m {
                        result.insert(k, v.to_map_recursive()?);
                    }
                    Ok(OwnedTerm::Map(result))
                } else {
                    Ok(map)
                }
            }
            OwnedTerm::List(elements) => {
                let converted: Result<Vec<OwnedTerm>, _> =
                    elements.iter().map(|v| v.to_map_recursive()).collect();
                Ok(OwnedTerm::List(converted?))
            }
            OwnedTerm::Map(m) => {
                let mut result = BTreeMap::new();
                for (k, v) in m {
                    result.insert(k.clone(), v.to_map_recursive()?);
                }
                Ok(OwnedTerm::Map(result))
            }
            OwnedTerm::Nil => Ok(OwnedTerm::List(vec![])),
            _ => Ok(self.clone()),
        }
    }

    pub fn atomize_keys(&self) -> Result<OwnedTerm, TermConversionError> {
        match self {
            OwnedTerm::List(elements) => {
                let converted: Vec<OwnedTerm> = elements
                    .iter()
                    .filter_map(|el| {
                        if let OwnedTerm::Tuple(t) = el
                            && t.len() == 2
                        {
                            let key = match &t[0] {
                                OwnedTerm::Atom(_) => t[0].clone(),
                                OwnedTerm::Binary(b) => {
                                    let s = String::from_utf8_lossy(b);
                                    OwnedTerm::Atom(Atom::new(s.as_ref()))
                                }
                                OwnedTerm::String(s) => OwnedTerm::Atom(Atom::new(s)),
                                _ => return None,
                            };
                            Some(OwnedTerm::Tuple(vec![key, t[1].clone()]))
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(OwnedTerm::List(converted))
            }
            OwnedTerm::Map(m) => {
                let mut result = BTreeMap::new();
                for (k, v) in m {
                    let key = match k {
                        OwnedTerm::Atom(_) => k.clone(),
                        OwnedTerm::Binary(b) => {
                            let s = String::from_utf8_lossy(b);
                            OwnedTerm::Atom(Atom::new(s.as_ref()))
                        }
                        OwnedTerm::String(s) => OwnedTerm::Atom(Atom::new(s)),
                        _ => continue,
                    };
                    result.insert(key, v.clone());
                }
                Ok(OwnedTerm::Map(result))
            }
            OwnedTerm::Nil => Ok(OwnedTerm::List(vec![])),
            _ => Err(TermConversionError::WrongType {
                expected: "List or Map",
                actual: self.type_name(),
            }),
        }
    }

    pub fn as_list_wrapped(&self) -> OwnedTerm {
        match self {
            OwnedTerm::List(_) | OwnedTerm::Nil => self.clone(),
            _ => OwnedTerm::List(vec![self.clone()]),
        }
    }

    pub fn proplist_iter(&self) -> Option<ProplistIter<'_>> {
        match self {
            OwnedTerm::List(elements) => Some(ProplistIter {
                iter: elements.iter(),
            }),
            OwnedTerm::Nil => Some(ProplistIter { iter: [].iter() }),
            _ => None,
        }
    }

    pub fn map_get_atom_key(&self, key: &str) -> Option<&OwnedTerm> {
        match self {
            OwnedTerm::Map(map) => map.iter().find_map(|(k, v)| {
                if let OwnedTerm::Atom(atom) = k
                    && atom.as_ref() == key
                {
                    return Some(v);
                }
                None
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn map_get_i64(&self, key: &str) -> Option<i64> {
        self.map_get_atom_key(key).and_then(|t| t.as_integer())
    }

    #[inline]
    pub fn map_get_i64_or(&self, key: &str, default: i64) -> i64 {
        self.map_get_i64(key).unwrap_or(default)
    }

    #[inline]
    pub fn map_get_bool(&self, key: &str) -> Option<bool> {
        self.map_get_atom_key(key).and_then(|t| t.as_bool())
    }

    #[inline]
    pub fn map_get_bool_or(&self, key: &str, default: bool) -> bool {
        self.map_get_bool(key).unwrap_or(default)
    }

    #[inline]
    pub fn map_get_atom(&self, key: &str) -> Option<&Atom> {
        self.map_get_atom_key(key).and_then(|t| t.as_atom())
    }

    #[inline]
    pub fn map_get_string(&self, key: &str) -> Option<String> {
        self.map_get_atom_key(key)
            .and_then(|t| t.as_erlang_string())
    }

    #[inline]
    pub fn map_get_string_or(&self, key: &str, default: &str) -> String {
        self.map_get_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    pub fn map_get_pid(&self, key: &str) -> Option<&ExternalPid> {
        self.map_get_atom_key(key).and_then(|t| t.as_pid())
    }

    #[inline]
    pub fn map_get_atom_string(&self, key: &str) -> Option<String> {
        self.map_get_atom(key).map(|a| a.to_string())
    }

    #[inline]
    pub fn map_get_atom_string_or(&self, key: &str, default: &str) -> String {
        self.map_get_atom_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    pub fn map_get_pid_string(&self, key: &str) -> Option<String> {
        self.map_get_pid(key).map(|p| p.to_string())
    }

    #[inline]
    pub fn map_get_pid_string_or(&self, key: &str, default: &str) -> String {
        self.map_get_pid_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    pub fn map_get_mfa_string(&self, key: &str) -> Option<String> {
        self.map_get_atom_key(key).and_then(|t| t.format_as_mfa())
    }

    #[inline]
    pub fn map_get_mfa_string_or(&self, key: &str, default: &str) -> String {
        self.map_get_mfa_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    pub fn as_erlang_string(&self) -> Option<String> {
        match self {
            OwnedTerm::List(integers) => {
                let bytes: Vec<u8> = integers
                    .iter()
                    .filter_map(|t| {
                        if let OwnedTerm::Integer(i) = t {
                            if *i >= 0 && *i <= 255 {
                                Some(*i as u8)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                if bytes.len() == integers.len() {
                    Some(String::from_utf8_lossy(&bytes).to_string())
                } else {
                    None
                }
            }
            OwnedTerm::String(s) => Some(s.clone()),
            OwnedTerm::Binary(b) => Some(String::from_utf8_lossy(b).to_string()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_erlang_string_or(&self, default: &str) -> String {
        self.as_erlang_string()
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    #[must_use]
    pub fn tuple_get(&self, index: usize) -> Option<&OwnedTerm> {
        match self {
            OwnedTerm::Tuple(t) => t.get(index),
            _ => None,
        }
    }

    #[inline]
    pub fn tuple_get_string(&self, index: usize) -> Option<String> {
        self.tuple_get(index).and_then(|t| t.as_erlang_string())
    }

    #[inline]
    pub fn tuple_get_string_or(&self, index: usize, default: &str) -> String {
        self.tuple_get_string(index)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    pub fn tuple_get_atom_string(&self, index: usize) -> Option<String> {
        self.tuple_get(index)
            .and_then(|t| t.as_atom())
            .map(|a| a.to_string())
    }

    #[inline]
    pub fn tuple_get_atom_string_or(&self, index: usize, default: &str) -> String {
        self.tuple_get_atom_string(index)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    #[must_use]
    pub fn charlist<S: AsRef<str>>(s: S) -> Self {
        let chars: Vec<OwnedTerm> = s
            .as_ref()
            .chars()
            .map(|c| OwnedTerm::Integer(c as i64))
            .collect();
        OwnedTerm::List(chars)
    }

    #[inline]
    #[must_use]
    pub fn is_charlist(&self) -> bool {
        fn is_valid_unicode_scalar(i: i64) -> bool {
            (0..=0x10FFFF).contains(&i) && !(0xD800..=0xDFFF).contains(&i)
        }
        match self {
            OwnedTerm::List(elements) => elements
                .iter()
                .all(|t| matches!(t, OwnedTerm::Integer(i) if is_valid_unicode_scalar(*i))),
            OwnedTerm::Nil => true,
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_charlist_string(&self) -> Option<String> {
        match self {
            OwnedTerm::List(elements) => {
                let chars: Option<String> = elements
                    .iter()
                    .map(|t| match t {
                        OwnedTerm::Integer(i) if *i >= 0 && *i <= 0x10FFFF => {
                            char::from_u32(*i as u32)
                        }
                        _ => None,
                    })
                    .collect();
                chars
            }
            OwnedTerm::Nil => Some(String::new()),
            OwnedTerm::String(s) => Some(s.clone()),
            OwnedTerm::Binary(b) => Some(String::from_utf8_lossy(b).to_string()),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_list_or_empty(&self) -> &[OwnedTerm] {
        match self {
            OwnedTerm::List(l) => l,
            OwnedTerm::Nil => &[],
            _ => &[],
        }
    }

    #[inline]
    #[must_use]
    pub fn try_as_mfa(&self) -> Option<Mfa> {
        Mfa::try_from_term(self)
    }

    #[inline]
    #[must_use]
    pub fn format_as_mfa(&self) -> Option<String> {
        self.try_as_mfa().map(|mfa| mfa.to_string())
    }

    #[inline]
    #[must_use]
    pub fn as_pid(&self) -> Option<&ExternalPid> {
        match self {
            OwnedTerm::Pid(pid) => Some(pid),
            _ => None,
        }
    }

    #[inline]
    pub fn try_as_pid(&self) -> Result<&ExternalPid, TermConversionError> {
        self.as_pid().ok_or(TermConversionError::WrongType {
            expected: "Pid",
            actual: self.type_name(),
        })
    }

    #[inline]
    #[must_use]
    pub fn is_pid(&self) -> bool {
        matches!(self, OwnedTerm::Pid(_))
    }

    #[inline]
    #[must_use]
    pub fn format_as_pid(&self) -> Option<String> {
        self.as_pid().map(|p| p.to_string())
    }

    #[inline]
    pub fn proplist_get_i64(&self, key: &str) -> Option<i64> {
        self.proplist_get_atom_key(key).and_then(|t| t.as_integer())
    }

    #[inline]
    pub fn proplist_get_i64_or(&self, key: &str, default: i64) -> i64 {
        self.proplist_get_i64(key).unwrap_or(default)
    }

    #[inline]
    pub fn proplist_get_bool(&self, key: &str) -> Option<bool> {
        self.proplist_get_atom_key(key).and_then(|t| t.as_bool())
    }

    #[inline]
    pub fn proplist_get_bool_or(&self, key: &str, default: bool) -> bool {
        self.proplist_get_bool(key).unwrap_or(default)
    }

    #[inline]
    pub fn proplist_get_atom(&self, key: &str) -> Option<&Atom> {
        self.proplist_get_atom_key(key).and_then(|t| t.as_atom())
    }

    #[inline]
    pub fn proplist_get_string(&self, key: &str) -> Option<String> {
        self.proplist_get_atom_key(key)
            .and_then(|t| t.as_erlang_string())
    }

    #[inline]
    pub fn proplist_get_string_or(&self, key: &str, default: &str) -> String {
        self.proplist_get_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    pub fn proplist_get_pid(&self, key: &str) -> Option<&ExternalPid> {
        self.proplist_get_atom_key(key).and_then(|t| t.as_pid())
    }

    #[inline]
    pub fn proplist_get_atom_string(&self, key: &str) -> Option<String> {
        self.proplist_get_atom(key).map(|a| a.to_string())
    }

    #[inline]
    pub fn proplist_get_atom_string_or(&self, key: &str, default: &str) -> String {
        self.proplist_get_atom_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    pub fn proplist_get_pid_string(&self, key: &str) -> Option<String> {
        self.proplist_get_pid(key).map(|p| p.to_string())
    }

    #[inline]
    pub fn proplist_get_pid_string_or(&self, key: &str, default: &str) -> String {
        self.proplist_get_pid_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    pub fn proplist_get_mfa_string(&self, key: &str) -> Option<String> {
        self.proplist_get_atom_key(key)
            .and_then(|t| t.format_as_mfa())
    }

    #[inline]
    pub fn proplist_get_mfa_string_or(&self, key: &str, default: &str) -> String {
        self.proplist_get_mfa_string(key)
            .unwrap_or_else(|| default.to_string())
    }

    #[inline]
    #[must_use]
    pub fn atom_list(names: &[&str]) -> Self {
        OwnedTerm::List(
            names
                .iter()
                .map(|n| OwnedTerm::Atom(Atom::new(*n)))
                .collect(),
        )
    }

    pub fn map_iter(&self) -> Option<impl Iterator<Item = (&OwnedTerm, &OwnedTerm)>> {
        match self {
            OwnedTerm::Map(m) => Some(m.iter()),
            _ => None,
        }
    }

    pub fn try_into_list(self) -> Result<Vec<OwnedTerm>, TermConversionError> {
        match self {
            OwnedTerm::List(l) => Ok(l),
            OwnedTerm::Nil => Ok(Vec::new()),
            _ => Err(TermConversionError::WrongType {
                expected: "List or Nil",
                actual: self.type_name(),
            }),
        }
    }

    pub fn try_into_tuple(self) -> Result<Vec<OwnedTerm>, TermConversionError> {
        match self {
            OwnedTerm::Tuple(t) => Ok(t),
            _ => Err(TermConversionError::WrongType {
                expected: "Tuple",
                actual: self.type_name(),
            }),
        }
    }

    pub fn try_into_map(self) -> Result<BTreeMap<OwnedTerm, OwnedTerm>, TermConversionError> {
        match self {
            OwnedTerm::Map(m) => Ok(m),
            _ => Err(TermConversionError::WrongType {
                expected: "Map",
                actual: self.type_name(),
            }),
        }
    }

    pub fn try_into_binary(self) -> Result<Vec<u8>, TermConversionError> {
        match self {
            OwnedTerm::Binary(b) => Ok(b),
            OwnedTerm::String(s) => Ok(s.into_bytes()),
            _ => Err(TermConversionError::WrongType {
                expected: "Binary or String",
                actual: self.type_name(),
            }),
        }
    }

    pub fn try_into_string(self) -> Result<String, TermConversionError> {
        match self {
            OwnedTerm::String(s) => Ok(s),
            OwnedTerm::Binary(b) => {
                String::from_utf8(b).map_err(|_| TermConversionError::OutOfRange)
            }
            _ => Err(TermConversionError::WrongType {
                expected: "String or Binary",
                actual: self.type_name(),
            }),
        }
    }

    pub fn try_into_atom(self) -> Result<Arc<str>, TermConversionError> {
        match self {
            OwnedTerm::Atom(a) => Ok(a.name),
            _ => Err(TermConversionError::WrongType {
                expected: "Atom",
                actual: self.type_name(),
            }),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            OwnedTerm::List(l) => l.len(),
            OwnedTerm::Tuple(t) => t.len(),
            OwnedTerm::Map(m) => m.len(),
            OwnedTerm::Binary(b) => b.len(),
            OwnedTerm::String(s) => s.len(),
            OwnedTerm::Nil => 0,
            _ => 0,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            OwnedTerm::List(l) => l.is_empty(),
            OwnedTerm::Tuple(t) => t.is_empty(),
            OwnedTerm::Map(m) => m.is_empty(),
            OwnedTerm::Binary(b) => b.is_empty(),
            OwnedTerm::String(s) => s.is_empty(),
            OwnedTerm::Nil => true,
            _ => false,
        }
    }

    pub fn estimated_encoded_size(&self) -> usize {
        match self {
            OwnedTerm::Atom(a) => 3 + a.len(),
            OwnedTerm::Integer(i) => {
                if (0..=255).contains(i) {
                    2
                } else if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                    5
                } else {
                    let abs = i.unsigned_abs();
                    let bytes = (64u32 - abs.leading_zeros()).div_ceil(8);
                    3 + bytes as usize
                }
            }
            OwnedTerm::Float(_) => 9,
            OwnedTerm::Binary(b) => 5 + b.len(),
            OwnedTerm::BitBinary { bytes, .. } => 6 + bytes.len(),
            OwnedTerm::String(s) => 5 + s.len(),
            OwnedTerm::List(l) => {
                5 + 1 + l.iter().map(|t| t.estimated_encoded_size()).sum::<usize>()
            }
            OwnedTerm::ImproperList { elements, tail } => {
                5 + elements
                    .iter()
                    .map(|t| t.estimated_encoded_size())
                    .sum::<usize>()
                    + tail.estimated_encoded_size()
            }
            OwnedTerm::Tuple(t) => {
                let base = if t.len() <= 255 { 2 } else { 5 };
                base + t.iter().map(|t| t.estimated_encoded_size()).sum::<usize>()
            }
            OwnedTerm::Map(m) => {
                5 + m
                    .iter()
                    .map(|(k, v)| k.estimated_encoded_size() + v.estimated_encoded_size())
                    .sum::<usize>()
            }
            OwnedTerm::Pid(_) => 17,
            OwnedTerm::Port(_) => 16,
            OwnedTerm::Reference(r) => 7 + r.ids.len() * 4,
            OwnedTerm::BigInt(b) => {
                let base = if b.digits.len() <= 255 { 2 } else { 5 };
                base + 1 + b.digits.len()
            }
            OwnedTerm::ExternalFun(_) => 32,
            OwnedTerm::InternalFun(f) => {
                64 + f
                    .free_vars
                    .iter()
                    .map(|t| t.estimated_encoded_size())
                    .sum::<usize>()
            }
            OwnedTerm::Nil => 1,
        }
    }

    /// Returns the elements of a 2-tuple, or None if not a 2-tuple.
    ///
    /// # Example
    /// ```
    /// use erltf::OwnedTerm;
    /// let term = OwnedTerm::Tuple(vec![OwnedTerm::atom("ok"), OwnedTerm::integer(42)]);
    /// if let Some((first, second)) = term.as_2_tuple() {
    ///     assert!(first.is_atom_with_name("ok"));
    ///     assert_eq!(second.as_integer(), Some(42));
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn as_2_tuple(&self) -> Option<(&OwnedTerm, &OwnedTerm)> {
        match self {
            OwnedTerm::Tuple(elements) if elements.len() == 2 => Some((&elements[0], &elements[1])),
            _ => None,
        }
    }

    /// Returns the elements of a 3-tuple, or None if not a 3-tuple.
    #[inline]
    #[must_use]
    pub fn as_3_tuple(&self) -> Option<(&OwnedTerm, &OwnedTerm, &OwnedTerm)> {
        match self {
            OwnedTerm::Tuple(elements) if elements.len() == 3 => {
                Some((&elements[0], &elements[1], &elements[2]))
            }
            _ => None,
        }
    }

    /// Returns the elements of a 4-tuple, or None if not a 4-tuple.
    #[inline]
    #[must_use]
    pub fn as_4_tuple(&self) -> Option<(&OwnedTerm, &OwnedTerm, &OwnedTerm, &OwnedTerm)> {
        match self {
            OwnedTerm::Tuple(elements) if elements.len() == 4 => {
                Some((&elements[0], &elements[1], &elements[2], &elements[3]))
            }
            _ => None,
        }
    }

    /// Returns the value from an `{:ok, value}` tuple, or None if not in that format.
    ///
    /// # Example
    /// ```
    /// use erltf::OwnedTerm;
    /// let term = OwnedTerm::ok_tuple(OwnedTerm::integer(42));
    /// assert_eq!(term.as_ok_tuple().and_then(|v| v.as_integer()), Some(42));
    /// ```
    #[inline]
    #[must_use]
    pub fn as_ok_tuple(&self) -> Option<&OwnedTerm> {
        match self.as_2_tuple() {
            Some((first, second)) if first.is_atom_with_name("ok") => Some(second),
            _ => None,
        }
    }

    /// Returns the reason from an `{:error, reason}` tuple, or None if not in that format.
    #[inline]
    #[must_use]
    pub fn as_error_tuple(&self) -> Option<&OwnedTerm> {
        match self.as_2_tuple() {
            Some((first, second)) if first.is_atom_with_name("error") => Some(second),
            _ => None,
        }
    }

    /// Converts an `{:ok, value}` or `{:error, reason}` tuple to a Result.
    ///
    /// # Example
    /// ```
    /// use erltf::OwnedTerm;
    /// let ok_term = OwnedTerm::ok_tuple(OwnedTerm::integer(42));
    /// assert!(ok_term.as_ok_error().is_ok());
    ///
    /// let err_term = OwnedTerm::error_tuple(OwnedTerm::atom("not_found"));
    /// assert!(err_term.as_ok_error().is_err());
    /// ```
    #[inline]
    pub fn as_ok_error(&self) -> Result<&OwnedTerm, &OwnedTerm> {
        match self.as_2_tuple() {
            Some((first, second)) if first.is_atom_with_name("ok") => Ok(second),
            Some((first, second)) if first.is_atom_with_name("error") => Err(second),
            _ => Err(self),
        }
    }

    /// Returns true if this is the Elixir `nil` atom.
    #[inline]
    #[must_use]
    pub fn is_elixir_nil(&self) -> bool {
        self.is_atom_with_name("nil")
    }

    /// Returns true if this is the Erlang `undefined` atom.
    #[inline]
    #[must_use]
    pub fn is_erlang_undefined(&self) -> bool {
        self.is_atom_with_name("undefined")
    }

    /// Returns true if this is either `nil` or `undefined`.
    #[inline]
    #[must_use]
    pub fn is_nil_like(&self) -> bool {
        self.is_elixir_nil() || self.is_erlang_undefined()
    }

    /// Creates the Elixir `nil` atom.
    #[inline]
    #[must_use]
    pub fn elixir_nil() -> Self {
        OwnedTerm::atom("nil")
    }

    /// Creates the Erlang `undefined` atom.
    #[inline]
    #[must_use]
    pub fn erlang_undefined() -> Self {
        OwnedTerm::atom("undefined")
    }

    /// Returns true if this is an atom matching one of the provided names.
    ///
    /// # Example
    /// ```
    /// use erltf::OwnedTerm;
    /// let term = OwnedTerm::atom("ok");
    /// assert!(term.is_atom_one_of(&["ok", "error", "noreply"]));
    /// assert!(!term.is_atom_one_of(&["stop", "continue"]));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_atom_one_of(&self, names: &[&str]) -> bool {
        match self {
            OwnedTerm::Atom(a) => names.iter().any(|&n| a == n),
            _ => false,
        }
    }

    /// Returns true if this term is an Elixir struct (a map with `__struct__` key).
    ///
    /// # Example
    /// ```
    /// use erltf::OwnedTerm;
    /// use std::collections::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(OwnedTerm::atom("__struct__"), OwnedTerm::atom("Elixir.MyApp.User"));
    /// map.insert(OwnedTerm::atom("name"), OwnedTerm::string("Alice".to_string()));
    /// let term = OwnedTerm::Map(map);
    ///
    /// assert!(term.is_elixir_struct());
    /// assert_eq!(term.elixir_struct_module(), Some("Elixir.MyApp.User"));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_elixir_struct(&self) -> bool {
        self.elixir_struct_module().is_some()
    }

    /// Returns the module name if this is an Elixir struct.
    #[inline]
    #[must_use]
    pub fn elixir_struct_module(&self) -> Option<&str> {
        match self {
            OwnedTerm::Map(m) => {
                let struct_key = OwnedTerm::atom("__struct__");
                m.get(&struct_key).and_then(|v| v.atom_name())
            }
            _ => None,
        }
    }

    /// Returns true if this is an Elixir exception (has both `__struct__` and `__exception__` keys).
    #[inline]
    #[must_use]
    pub fn is_elixir_exception(&self) -> bool {
        match self {
            OwnedTerm::Map(m) => {
                let struct_key = OwnedTerm::atom("__struct__");
                let exception_key = OwnedTerm::atom("__exception__");
                m.contains_key(&struct_key) && m.get(&exception_key).is_some_and(|v| v.is_true())
            }
            _ => false,
        }
    }

    /// Formats the term in Elixir inspect style for debugging.
    ///
    /// # Example
    /// ```
    /// use erltf::OwnedTerm;
    /// assert_eq!(OwnedTerm::atom("ok").inspect(), ":ok");
    /// assert_eq!(OwnedTerm::integer(42).inspect(), "42");
    /// ```
    #[must_use]
    pub fn inspect(&self) -> String {
        self.inspect_impl(0)
    }

    fn inspect_impl(&self, depth: usize) -> String {
        if depth > 10 {
            return "...".to_string();
        }

        match self {
            OwnedTerm::Atom(a) => {
                let name = a.as_str();
                if name.starts_with("Elixir.") {
                    name.strip_prefix("Elixir.").unwrap_or(name).to_string()
                } else if name == "true" || name == "false" || name == "nil" {
                    name.to_string()
                } else if name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.is_empty()
                {
                    format!(":{name}")
                } else {
                    format!(":\"{}\"", name.replace('\"', "\\\""))
                }
            }
            OwnedTerm::Integer(i) => i.to_string(),
            OwnedTerm::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{f:.1}")
                } else {
                    f.to_string()
                }
            }
            OwnedTerm::Binary(b) => match std::str::from_utf8(b) {
                Ok(s) if s.chars().all(|c| !c.is_control() || c == '\n' || c == '\t') => {
                    format!("\"{}\"", s.replace('\"', "\\\""))
                }
                _ => {
                    let bytes: Vec<String> = b.iter().take(10).map(|b| b.to_string()).collect();
                    if b.len() > 10 {
                        format!("<<{}, ...>>", bytes.join(", "))
                    } else {
                        format!("<<{}>>", bytes.join(", "))
                    }
                }
            },
            OwnedTerm::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
            OwnedTerm::List(elements) => {
                if elements.is_empty() {
                    "[]".to_string()
                } else if self.is_charlist() {
                    if let Some(s) = self.as_charlist_string() {
                        return format!("~c\"{s}\"");
                    }
                    let items: Vec<String> = elements
                        .iter()
                        .take(20)
                        .map(|e| e.inspect_impl(depth + 1))
                        .collect();
                    if elements.len() > 20 {
                        format!("[{}, ...]", items.join(", "))
                    } else {
                        format!("[{}]", items.join(", "))
                    }
                } else if self.is_proplist() {
                    let items: Vec<String> = elements
                        .iter()
                        .take(20)
                        .filter_map(|e| {
                            e.as_2_tuple().map(|(k, v)| {
                                if let Some(name) = k.atom_name() {
                                    format!("{}: {}", name, v.inspect_impl(depth + 1))
                                } else {
                                    format!(
                                        "{{{}, {}}}",
                                        k.inspect_impl(depth + 1),
                                        v.inspect_impl(depth + 1)
                                    )
                                }
                            })
                        })
                        .collect();
                    if elements.len() > 20 {
                        format!("[{}, ...]", items.join(", "))
                    } else {
                        format!("[{}]", items.join(", "))
                    }
                } else {
                    let items: Vec<String> = elements
                        .iter()
                        .take(20)
                        .map(|e| e.inspect_impl(depth + 1))
                        .collect();
                    if elements.len() > 20 {
                        format!("[{}, ...]", items.join(", "))
                    } else {
                        format!("[{}]", items.join(", "))
                    }
                }
            }
            OwnedTerm::Tuple(elements) => {
                let items: Vec<String> =
                    elements.iter().map(|e| e.inspect_impl(depth + 1)).collect();
                format!("{{{}}}", items.join(", "))
            }
            OwnedTerm::Map(m) => {
                if let Some(module) = self.elixir_struct_module() {
                    let module_name = module.strip_prefix("Elixir.").unwrap_or(module);
                    let fields: Vec<String> = m
                        .iter()
                        .filter(|(k, _)| {
                            !k.is_atom_with_name("__struct__")
                                && !k.is_atom_with_name("__exception__")
                        })
                        .take(20)
                        .map(|(k, v)| {
                            if let Some(name) = k.atom_name() {
                                format!("{}: {}", name, v.inspect_impl(depth + 1))
                            } else {
                                format!(
                                    "{} => {}",
                                    k.inspect_impl(depth + 1),
                                    v.inspect_impl(depth + 1)
                                )
                            }
                        })
                        .collect();
                    format!("%{}{{{}}}", module_name, fields.join(", "))
                } else {
                    let items: Vec<String> = m
                        .iter()
                        .take(20)
                        .map(|(k, v)| {
                            if let Some(name) = k.atom_name() {
                                format!("{}: {}", name, v.inspect_impl(depth + 1))
                            } else {
                                format!(
                                    "{} => {}",
                                    k.inspect_impl(depth + 1),
                                    v.inspect_impl(depth + 1)
                                )
                            }
                        })
                        .collect();
                    if m.len() > 20 {
                        format!("%{{{}, ...}}", items.join(", "))
                    } else {
                        format!("%{{{}}}", items.join(", "))
                    }
                }
            }
            OwnedTerm::Pid(p) => format!("#PID<{}>", p),
            OwnedTerm::Port(p) => format!("#Port<{:?}>", p),
            OwnedTerm::Reference(r) => format!("#Reference<{:?}>", r),
            OwnedTerm::BigInt(b) => {
                let sign = if b.sign.is_negative() { "-" } else { "" };
                format!("{sign}<BigInt({} bytes)>", b.digits.len())
            }
            OwnedTerm::ExternalFun(f) => format!("&{}.{}/{}", f.module, f.function, f.arity),
            OwnedTerm::InternalFun(f) => format!("#Function<{}/{}>", f.module, f.arity),
            OwnedTerm::ImproperList { elements, tail } => {
                let items: Vec<String> =
                    elements.iter().map(|e| e.inspect_impl(depth + 1)).collect();
                format!("[{} | {}]", items.join(", "), tail.inspect_impl(depth + 1))
            }
            OwnedTerm::BitBinary { bytes, bits } => {
                format!("<<{} bytes, {} bits>>", bytes.len(), bits)
            }
            OwnedTerm::Nil => "[]".to_string(),
        }
    }
}

impl From<Atom> for OwnedTerm {
    fn from(a: Atom) -> Self {
        OwnedTerm::Atom(a)
    }
}

impl From<i64> for OwnedTerm {
    fn from(i: i64) -> Self {
        OwnedTerm::Integer(i)
    }
}

impl From<i32> for OwnedTerm {
    fn from(i: i32) -> Self {
        OwnedTerm::Integer(i as i64)
    }
}

impl From<i16> for OwnedTerm {
    fn from(i: i16) -> Self {
        OwnedTerm::Integer(i as i64)
    }
}

impl From<i8> for OwnedTerm {
    fn from(i: i8) -> Self {
        OwnedTerm::Integer(i as i64)
    }
}

impl From<u32> for OwnedTerm {
    fn from(i: u32) -> Self {
        OwnedTerm::Integer(i as i64)
    }
}

impl From<u16> for OwnedTerm {
    fn from(i: u16) -> Self {
        OwnedTerm::Integer(i as i64)
    }
}

impl From<u8> for OwnedTerm {
    fn from(i: u8) -> Self {
        OwnedTerm::Integer(i as i64)
    }
}

impl From<bool> for OwnedTerm {
    fn from(b: bool) -> Self {
        OwnedTerm::boolean(b)
    }
}

impl From<f32> for OwnedTerm {
    fn from(f: f32) -> Self {
        OwnedTerm::Float(f as f64)
    }
}

impl From<f64> for OwnedTerm {
    fn from(f: f64) -> Self {
        OwnedTerm::Float(f)
    }
}

impl From<Vec<u8>> for OwnedTerm {
    fn from(b: Vec<u8>) -> Self {
        OwnedTerm::Binary(b)
    }
}

impl From<String> for OwnedTerm {
    fn from(s: String) -> Self {
        OwnedTerm::String(s)
    }
}

impl From<&str> for OwnedTerm {
    fn from(s: &str) -> Self {
        OwnedTerm::String(s.to_string())
    }
}

impl From<Vec<Self>> for OwnedTerm {
    fn from(v: Vec<Self>) -> Self {
        OwnedTerm::List(v)
    }
}

impl From<BTreeMap<Self, Self>> for OwnedTerm {
    fn from(m: BTreeMap<Self, Self>) -> Self {
        OwnedTerm::Map(m)
    }
}

impl<K: Into<OwnedTerm>, V: Into<OwnedTerm>, S: BuildHasher> From<HashMap<K, V, S>> for OwnedTerm {
    fn from(m: HashMap<K, V, S>) -> Self {
        let map = m.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        OwnedTerm::Map(map)
    }
}

impl<T: Into<OwnedTerm> + Clone> From<&[T]> for OwnedTerm {
    fn from(slice: &[T]) -> Self {
        OwnedTerm::List(slice.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<OwnedTerm>> FromIterator<T> for OwnedTerm {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        OwnedTerm::List(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<OwnedTerm>, V: Into<OwnedTerm>> FromIterator<(K, V)> for OwnedTerm {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        OwnedTerm::Map(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl From<Vec<i32>> for OwnedTerm {
    fn from(v: Vec<i32>) -> Self {
        OwnedTerm::List(
            v.into_iter()
                .map(|i| OwnedTerm::Integer(i as i64))
                .collect(),
        )
    }
}

impl From<Vec<i64>> for OwnedTerm {
    fn from(v: Vec<i64>) -> Self {
        OwnedTerm::List(v.into_iter().map(OwnedTerm::Integer).collect())
    }
}

impl TryFrom<OwnedTerm> for i64 {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        match term {
            OwnedTerm::Integer(i) => Ok(i),
            _ => Err(TermConversionError::WrongType {
                expected: "Integer",
                actual: term.type_name(),
            }),
        }
    }
}

impl TryFrom<OwnedTerm> for f64 {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        match term {
            OwnedTerm::Float(f) => Ok(f),
            _ => Err(TermConversionError::WrongType {
                expected: "Float",
                actual: term.type_name(),
            }),
        }
    }
}

impl TryFrom<OwnedTerm> for String {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        match term {
            OwnedTerm::String(s) => Ok(s),
            OwnedTerm::Binary(b) => {
                String::from_utf8(b).map_err(|_| TermConversionError::OutOfRange)
            }
            _ => Err(TermConversionError::WrongType {
                expected: "String or Binary",
                actual: term.type_name(),
            }),
        }
    }
}

impl TryFrom<OwnedTerm> for Vec<u8> {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        match term {
            OwnedTerm::Binary(b) => Ok(b),
            OwnedTerm::String(s) => Ok(s.into_bytes()),
            _ => Err(TermConversionError::WrongType {
                expected: "Binary or String",
                actual: term.type_name(),
            }),
        }
    }
}

impl TryFrom<OwnedTerm> for Vec<OwnedTerm> {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        match term {
            OwnedTerm::List(l) => Ok(l),
            OwnedTerm::Tuple(t) => Ok(t),
            OwnedTerm::Nil => Ok(Vec::new()),
            _ => Err(TermConversionError::WrongType {
                expected: "List, Tuple, or Nil",
                actual: term.type_name(),
            }),
        }
    }
}

impl TryFrom<OwnedTerm> for bool {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        term.as_bool().ok_or(TermConversionError::WrongType {
            expected: "boolean atom (true/false)",
            actual: term.type_name(),
        })
    }
}

impl TryFrom<OwnedTerm> for u32 {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        match term {
            OwnedTerm::Integer(i) if i >= 0 && i <= u32::MAX as i64 => Ok(i as u32),
            OwnedTerm::Integer(_) => Err(TermConversionError::OutOfRange),
            _ => Err(TermConversionError::WrongType {
                expected: "Integer",
                actual: term.type_name(),
            }),
        }
    }
}

impl TryFrom<OwnedTerm> for u16 {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        match term {
            OwnedTerm::Integer(i) if i >= 0 && i <= u16::MAX as i64 => Ok(i as u16),
            OwnedTerm::Integer(_) => Err(TermConversionError::OutOfRange),
            _ => Err(TermConversionError::WrongType {
                expected: "Integer",
                actual: term.type_name(),
            }),
        }
    }
}

impl TryFrom<OwnedTerm> for u8 {
    type Error = TermConversionError;

    fn try_from(term: OwnedTerm) -> Result<Self, Self::Error> {
        match term {
            OwnedTerm::Integer(i) if i >= 0 && i <= u8::MAX as i64 => Ok(i as u8),
            OwnedTerm::Integer(_) => Err(TermConversionError::OutOfRange),
            _ => Err(TermConversionError::WrongType {
                expected: "Integer",
                actual: term.type_name(),
            }),
        }
    }
}

impl Hash for OwnedTerm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        discriminant(self).hash(state);

        match self {
            OwnedTerm::Atom(a) => a.hash(state),
            OwnedTerm::Integer(i) => i.hash(state),
            OwnedTerm::Binary(b) => b.hash(state),
            OwnedTerm::String(s) => s.hash(state),
            OwnedTerm::Pid(p) => p.hash(state),
            OwnedTerm::Port(p) => p.hash(state),
            OwnedTerm::Reference(r) => r.hash(state),
            OwnedTerm::Nil => (),
            OwnedTerm::Float(f) => f.to_bits().hash(state),
            OwnedTerm::BigInt(big) => big.hash(state),
            OwnedTerm::BitBinary { bytes, bits } => {
                bytes.hash(state);
                bits.hash(state);
            }
            OwnedTerm::List(elements) => {
                elements.len().hash(state);
                for elem in elements {
                    elem.hash(state);
                }
            }
            OwnedTerm::ImproperList { elements, tail } => {
                elements.len().hash(state);
                for elem in elements {
                    elem.hash(state);
                }
                tail.hash(state);
            }
            OwnedTerm::Tuple(elements) => {
                elements.len().hash(state);
                for elem in elements {
                    elem.hash(state);
                }
            }
            OwnedTerm::Map(map) => {
                map.len().hash(state);
                for (k, v) in map.iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
            OwnedTerm::ExternalFun(f) => f.hash(state),
            OwnedTerm::InternalFun(f) => {
                f.arity.hash(state);
                f.uniq.hash(state);
                f.index.hash(state);
                f.num_free.hash(state);
                f.module.hash(state);
                f.old_index.hash(state);
                f.old_uniq.hash(state);
                f.pid.hash(state);
                for var in &f.free_vars {
                    var.hash(state);
                }
            }
        }
    }
}

/// Returns the Erlang type ordering value for a term.
/// Erlang ordering: numbers < atoms < references < funs < ports < pids < tuples < maps < lists < binaries
const fn term_type_order(t: &OwnedTerm) -> u8 {
    match t {
        OwnedTerm::Integer(_) | OwnedTerm::BigInt(_) | OwnedTerm::Float(_) => 0,
        OwnedTerm::Atom(_) => 1,
        OwnedTerm::Reference(_) => 2,
        OwnedTerm::ExternalFun(_) | OwnedTerm::InternalFun(_) => 3,
        OwnedTerm::Port(_) => 4,
        OwnedTerm::Pid(_) => 5,
        OwnedTerm::Tuple(_) => 6,
        OwnedTerm::Map(_) => 7,
        OwnedTerm::Nil | OwnedTerm::List(_) | OwnedTerm::ImproperList { .. } => 8,
        OwnedTerm::Binary(_) | OwnedTerm::BitBinary { .. } | OwnedTerm::String(_) => 9,
    }
}

impl Eq for OwnedTerm {}

impl Ord for OwnedTerm {
    fn cmp(&self, other: &Self) -> Ordering {
        if discriminant(self) == discriminant(other) {
            match (self, other) {
                (OwnedTerm::Integer(a), OwnedTerm::Integer(b)) => return a.cmp(b),
                (OwnedTerm::Atom(a), OwnedTerm::Atom(b)) => return a.name.cmp(&b.name),
                (OwnedTerm::Binary(a), OwnedTerm::Binary(b)) => return a.cmp(b),
                (OwnedTerm::String(a), OwnedTerm::String(b)) => return a.cmp(b),
                (OwnedTerm::Nil, OwnedTerm::Nil) => return Ordering::Equal,
                _ => {}
            }
        }

        match term_type_order(self).cmp(&term_type_order(other)) {
            Ordering::Equal => match (self, other) {
                (OwnedTerm::Integer(a), OwnedTerm::Integer(b)) => a.cmp(b),
                (OwnedTerm::Integer(a), OwnedTerm::BigInt(b)) => compare_int_bigint(*a, b),
                (OwnedTerm::BigInt(a), OwnedTerm::Integer(b)) => compare_bigint_int(a, *b),
                (OwnedTerm::BigInt(a), OwnedTerm::BigInt(b)) => compare_bigint(a, b),
                (OwnedTerm::Integer(a), OwnedTerm::Float(b)) => compare_int_float(*a, *b),
                (OwnedTerm::Float(a), OwnedTerm::Integer(b)) => compare_float_int(*a, *b),
                (OwnedTerm::BigInt(a), OwnedTerm::Float(b)) => compare_bigint_float(a, *b),
                (OwnedTerm::Float(a), OwnedTerm::BigInt(b)) => compare_float_bigint(*a, b),
                (OwnedTerm::Float(a), OwnedTerm::Float(b)) => {
                    if a.is_nan() && b.is_nan() {
                        Ordering::Equal
                    } else if a.is_nan() {
                        Ordering::Greater
                    } else if b.is_nan() {
                        Ordering::Less
                    } else {
                        a.partial_cmp(b).unwrap_or(Ordering::Equal)
                    }
                }
                (OwnedTerm::Atom(a), OwnedTerm::Atom(b)) => a.name.cmp(&b.name),
                (OwnedTerm::Reference(a), OwnedTerm::Reference(b)) => a
                    .node
                    .name
                    .cmp(&b.node.name)
                    .then_with(|| a.creation.cmp(&b.creation))
                    .then_with(|| a.ids.cmp(&b.ids)),
                (OwnedTerm::ExternalFun(a), OwnedTerm::ExternalFun(b)) => a
                    .module
                    .name
                    .cmp(&b.module.name)
                    .then_with(|| a.function.name.cmp(&b.function.name))
                    .then_with(|| a.arity.cmp(&b.arity)),
                (OwnedTerm::InternalFun(a), OwnedTerm::InternalFun(b)) => a
                    .module
                    .name
                    .cmp(&b.module.name)
                    .then_with(|| a.old_index.cmp(&b.old_index))
                    .then_with(|| a.old_uniq.cmp(&b.old_uniq))
                    .then_with(|| a.index.cmp(&b.index))
                    .then_with(|| a.uniq.cmp(&b.uniq))
                    .then_with(|| a.pid.cmp(&b.pid))
                    .then_with(|| compare_term_lists(&a.free_vars, &b.free_vars)),
                (OwnedTerm::ExternalFun(_), OwnedTerm::InternalFun(_)) => Ordering::Less,
                (OwnedTerm::InternalFun(_), OwnedTerm::ExternalFun(_)) => Ordering::Greater,
                (OwnedTerm::Port(a), OwnedTerm::Port(b)) => a
                    .node
                    .name
                    .cmp(&b.node.name)
                    .then_with(|| a.id.cmp(&b.id))
                    .then_with(|| a.creation.cmp(&b.creation)),
                (OwnedTerm::Pid(a), OwnedTerm::Pid(b)) => a
                    .node
                    .name
                    .cmp(&b.node.name)
                    .then_with(|| a.id.cmp(&b.id))
                    .then_with(|| a.serial.cmp(&b.serial))
                    .then_with(|| a.creation.cmp(&b.creation)),
                (OwnedTerm::Tuple(a), OwnedTerm::Tuple(b)) => {
                    a.len().cmp(&b.len()).then_with(|| {
                        for (x, y) in a.iter().zip(b.iter()) {
                            match x.cmp(y) {
                                Ordering::Equal => continue,
                                other => return other,
                            }
                        }
                        Ordering::Equal
                    })
                }
                (OwnedTerm::Map(a), OwnedTerm::Map(b)) => a.len().cmp(&b.len()).then_with(|| {
                    for ((k1, v1), (k2, v2)) in a.iter().zip(b.iter()) {
                        match k1.cmp(k2) {
                            Ordering::Equal => match v1.cmp(v2) {
                                Ordering::Equal => continue,
                                other => return other,
                            },
                            other => return other,
                        }
                    }
                    Ordering::Equal
                }),
                (OwnedTerm::Nil, OwnedTerm::Nil) => Ordering::Equal,
                (OwnedTerm::List(a), OwnedTerm::List(b)) => {
                    for (x, y) in a.iter().zip(b.iter()) {
                        match x.cmp(y) {
                            Ordering::Equal => continue,
                            other => return other,
                        }
                    }
                    a.len().cmp(&b.len())
                }
                (OwnedTerm::List(a), OwnedTerm::Nil) => {
                    if a.is_empty() {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                }
                (OwnedTerm::Nil, OwnedTerm::List(b)) => {
                    if b.is_empty() {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                }
                (
                    OwnedTerm::ImproperList {
                        elements: a,
                        tail: ta,
                    },
                    OwnedTerm::ImproperList {
                        elements: b,
                        tail: tb,
                    },
                ) => {
                    for (x, y) in a.iter().zip(b.iter()) {
                        match x.cmp(y) {
                            Ordering::Equal => continue,
                            other => return other,
                        }
                    }
                    a.len().cmp(&b.len()).then_with(|| ta.cmp(tb))
                }
                (OwnedTerm::Binary(a), OwnedTerm::Binary(b)) => a.cmp(b),
                (OwnedTerm::String(a), OwnedTerm::String(b)) => a.cmp(b),
                (OwnedTerm::Binary(a), OwnedTerm::String(b)) => a.as_slice().cmp(b.as_bytes()),
                (OwnedTerm::String(a), OwnedTerm::Binary(b)) => a.as_bytes().cmp(b.as_slice()),
                (
                    OwnedTerm::BitBinary {
                        bytes: a,
                        bits: abits,
                    },
                    OwnedTerm::BitBinary {
                        bytes: b,
                        bits: bbits,
                    },
                ) => a.cmp(b).then_with(|| abits.cmp(bbits)),
                _ => Ordering::Equal,
            },
            other => other,
        }
    }
}

impl PartialOrd for OwnedTerm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for OwnedTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OwnedTerm::Atom(a) => write!(f, "{}", a.name),
            OwnedTerm::Integer(i) => write!(f, "{}", i),
            OwnedTerm::Float(fl) => write!(f, "{}", fl),
            OwnedTerm::Binary(b) => write!(f, "<<{} bytes>>", b.len()),
            OwnedTerm::BitBinary { bytes, bits } => {
                write!(f, "<<{} bytes, {} bits>>", bytes.len(), bits)
            }
            OwnedTerm::String(s) => write!(f, "\"{}\"", s),
            OwnedTerm::List(l) => {
                write!(f, "[")?;
                for (i, term) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", term)?;
                }
                write!(f, "]")
            }
            OwnedTerm::Tuple(t) => {
                write!(f, "{{")?;
                for (i, term) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", term)?;
                }
                write!(f, "}}")
            }
            OwnedTerm::Map(m) => {
                write!(f, "#{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} => {}", k, v)?;
                }
                write!(f, "}}")
            }
            OwnedTerm::Nil => write!(f, "[]"),
            OwnedTerm::Pid(p) => write!(f, "<{}.{}.{}>", p.id, p.serial, p.creation),
            OwnedTerm::Port(p) => write!(f, "#Port<{}>", p.id),
            OwnedTerm::Reference(r) => write!(f, "#Ref<{:?}>", r.ids),
            OwnedTerm::BigInt(big) => {
                let sign = if big.sign.is_negative() { "-" } else { "" };
                write!(f, "{}BigInt<{} bytes>", sign, big.digits.len())
            }
            OwnedTerm::ExternalFun(fun) => write!(
                f,
                "fun {}:{}/{}",
                fun.module.name, fun.function.name, fun.arity
            ),
            OwnedTerm::InternalFun(fun) => write!(f, "fun {}/{}", fun.module.name, fun.arity),
            OwnedTerm::ImproperList { elements, tail } => {
                write!(f, "[")?;
                for (i, term) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", term)?;
                }
                write!(f, " | {}]", tail)
            }
        }
    }
}

pub enum OwnedTermIter<'a> {
    Slice(std::slice::Iter<'a, OwnedTerm>),
    Empty,
}

impl<'a> Iterator for OwnedTermIter<'a> {
    type Item = &'a OwnedTerm;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OwnedTermIter::Slice(iter) => iter.next(),
            OwnedTermIter::Empty => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            OwnedTermIter::Slice(iter) => iter.size_hint(),
            OwnedTermIter::Empty => (0, Some(0)),
        }
    }
}

impl<'a> ExactSizeIterator for OwnedTermIter<'a> {
    fn len(&self) -> usize {
        match self {
            OwnedTermIter::Slice(iter) => iter.len(),
            OwnedTermIter::Empty => 0,
        }
    }
}

impl IntoIterator for OwnedTerm {
    type Item = OwnedTerm;
    type IntoIter = OwnedTermIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            OwnedTerm::List(elements) | OwnedTerm::Tuple(elements) => {
                OwnedTermIntoIter::Vec(elements.into_iter())
            }
            OwnedTerm::Nil => OwnedTermIntoIter::Empty,
            _ => OwnedTermIntoIter::Empty,
        }
    }
}

pub enum OwnedTermIntoIter {
    Vec(std::vec::IntoIter<OwnedTerm>),
    Empty,
}

impl Iterator for OwnedTermIntoIter {
    type Item = OwnedTerm;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OwnedTermIntoIter::Vec(iter) => iter.next(),
            OwnedTermIntoIter::Empty => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            OwnedTermIntoIter::Vec(iter) => iter.size_hint(),
            OwnedTermIntoIter::Empty => (0, Some(0)),
        }
    }
}

impl ExactSizeIterator for OwnedTermIntoIter {
    fn len(&self) -> usize {
        match self {
            OwnedTermIntoIter::Vec(iter) => iter.len(),
            OwnedTermIntoIter::Empty => 0,
        }
    }
}

pub struct ProplistIter<'a> {
    iter: std::slice::Iter<'a, OwnedTerm>,
}

impl<'a> Iterator for ProplistIter<'a> {
    type Item = (&'a OwnedTerm, &'a OwnedTerm);

    fn next(&mut self) -> Option<Self::Item> {
        for element in self.iter.by_ref() {
            match element {
                OwnedTerm::Tuple(t) if t.len() == 2 => {
                    return Some((&t[0], &t[1]));
                }
                OwnedTerm::Atom(_) => {
                    static TRUE_ATOM: OnceLock<OwnedTerm> = OnceLock::new();
                    let true_val = TRUE_ATOM.get_or_init(|| OwnedTerm::boolean(true));
                    return Some((element, true_val));
                }
                _ => continue,
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.iter.len()))
    }
}

impl Index<usize> for OwnedTerm {
    type Output = OwnedTerm;

    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            OwnedTerm::List(elements) | OwnedTerm::Tuple(elements) => &elements[index],
            OwnedTerm::Nil => panic!(
                "index out of bounds: the len is 0 but the index is {}",
                index
            ),
            _ => panic!("cannot index into {}", self.type_name()),
        }
    }
}

impl Index<&OwnedTerm> for OwnedTerm {
    type Output = OwnedTerm;

    #[track_caller]
    fn index(&self, key: &OwnedTerm) -> &Self::Output {
        match self {
            OwnedTerm::Map(m) => m.get(key).unwrap_or_else(|| panic!("key not found in map")),
            _ => panic!("cannot index {} with a key", self.type_name()),
        }
    }
}

pub trait TermIndex {
    fn get_from_term<'a>(&self, term: &'a OwnedTerm) -> Option<&'a OwnedTerm>;
}

impl TermIndex for usize {
    fn get_from_term<'a>(&self, term: &'a OwnedTerm) -> Option<&'a OwnedTerm> {
        match term {
            OwnedTerm::List(elements) | OwnedTerm::Tuple(elements) => elements.get(*self),
            _ => None,
        }
    }
}

impl TermIndex for &OwnedTerm {
    fn get_from_term<'a>(&self, term: &'a OwnedTerm) -> Option<&'a OwnedTerm> {
        match term {
            OwnedTerm::Map(m) => m.get(self),
            _ => None,
        }
    }
}

impl TermIndex for &str {
    fn get_from_term<'a>(&self, term: &'a OwnedTerm) -> Option<&'a OwnedTerm> {
        match term {
            OwnedTerm::Map(m) => {
                let key = OwnedTerm::atom(*self);
                m.get(&key)
            }
            _ => None,
        }
    }
}

pub struct MapBuilder {
    map: BTreeMap<OwnedTerm, OwnedTerm>,
}

impl MapBuilder {
    pub fn new() -> Self {
        MapBuilder {
            map: BTreeMap::new(),
        }
    }

    pub fn insert<K: Into<OwnedTerm>, V: Into<OwnedTerm>>(
        &mut self,
        key: K,
        value: V,
    ) -> &mut Self {
        self.map.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> OwnedTerm {
        OwnedTerm::Map(self.map)
    }
}

impl Default for MapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ListBuilder {
    elements: Vec<OwnedTerm>,
}

impl ListBuilder {
    pub fn new() -> Self {
        ListBuilder {
            elements: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        ListBuilder {
            elements: Vec::with_capacity(capacity),
        }
    }

    pub fn push<T: Into<OwnedTerm>>(&mut self, element: T) -> &mut Self {
        self.elements.push(element.into());
        self
    }

    pub fn extend<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<OwnedTerm>,
    {
        self.elements.extend(iter.into_iter().map(Into::into));
        self
    }

    pub fn build(self) -> OwnedTerm {
        OwnedTerm::List(self.elements)
    }

    pub fn build_tuple(self) -> OwnedTerm {
        OwnedTerm::Tuple(self.elements)
    }
}

impl Default for ListBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OwnedTerm {
    pub fn map_builder() -> MapBuilder {
        MapBuilder::new()
    }

    pub fn list_builder() -> ListBuilder {
        ListBuilder::new()
    }
}

fn compare_int_bigint(i: i64, big: &BigInt) -> Ordering {
    if big.digits.is_empty() {
        return i.cmp(&0);
    }

    if big.sign.is_negative() {
        if i >= 0 {
            return Ordering::Greater;
        }
        if big.digits.len() > 8 {
            return Ordering::Greater;
        }
        let abs_i = i.wrapping_neg() as u64;
        let big_val = bigint_to_u64(big);
        abs_i.cmp(&big_val).reverse()
    } else {
        if i < 0 {
            return Ordering::Less;
        }
        if big.digits.len() > 8 {
            return Ordering::Less;
        }
        let abs_i = i as u64;
        let big_val = bigint_to_u64(big);
        abs_i.cmp(&big_val)
    }
}

fn compare_bigint_int(big: &BigInt, i: i64) -> Ordering {
    compare_int_bigint(i, big).reverse()
}

fn compare_bigint(a: &BigInt, b: &BigInt) -> Ordering {
    match (a.sign, b.sign) {
        (Sign::Positive, Sign::Negative) => Ordering::Greater,
        (Sign::Negative, Sign::Positive) => Ordering::Less,
        (Sign::Positive, Sign::Positive) => a
            .digits
            .len()
            .cmp(&b.digits.len())
            .then_with(|| a.digits.cmp(&b.digits)),
        (Sign::Negative, Sign::Negative) => a
            .digits
            .len()
            .cmp(&b.digits.len())
            .then_with(|| a.digits.cmp(&b.digits))
            .reverse(),
    }
}

fn bigint_to_u64(big: &BigInt) -> u64 {
    let mut result = 0u64;
    for (i, &byte) in big.digits.iter().enumerate().take(8) {
        result |= (byte as u64) << (i * 8);
    }
    result
}

fn compare_int_float(i: i64, f: f64) -> Ordering {
    if f.is_nan() {
        return Ordering::Less;
    }
    let i_as_f = i as f64;
    i_as_f.partial_cmp(&f).unwrap_or(Ordering::Equal)
}

fn compare_float_int(f: f64, i: i64) -> Ordering {
    compare_int_float(i, f).reverse()
}

fn compare_bigint_float(big: &BigInt, f: f64) -> Ordering {
    if f.is_nan() {
        return Ordering::Less;
    }
    let big_as_f = bigint_to_f64(big);
    big_as_f.partial_cmp(&f).unwrap_or(Ordering::Equal)
}

fn compare_float_bigint(f: f64, big: &BigInt) -> Ordering {
    compare_bigint_float(big, f).reverse()
}

fn bigint_to_f64(big: &BigInt) -> f64 {
    let mut result = 0f64;
    let mut scale = 1.0f64;

    for &byte in big.digits.iter() {
        let contribution = (byte as f64) * scale;
        if contribution.is_infinite() || scale.is_infinite() {
            return if big.sign.is_negative() {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        result += contribution;
        scale *= 256.0;
    }

    if big.sign.is_negative() {
        -result
    } else {
        result
    }
}

fn compare_term_lists(a: &[OwnedTerm], b: &[OwnedTerm]) -> Ordering {
    for (x, y) in a.iter().zip(b.iter()) {
        match x.cmp(y) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    a.len().cmp(&b.len())
}

#[cfg(feature = "serde")]
impl Serialize for OwnedTerm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OwnedTerm::Atom(atom) => match atom.as_str() {
                "true" => serializer.serialize_bool(true),
                "false" => serializer.serialize_bool(false),
                "nil" | "undefined" => serializer.serialize_none(),
                s => serializer.serialize_str(s),
            },
            OwnedTerm::Integer(i) => serializer.serialize_i64(*i),
            OwnedTerm::Float(f) => serializer.serialize_f64(*f),
            OwnedTerm::Binary(b) => serializer.serialize_bytes(b),
            OwnedTerm::String(s) => serializer.serialize_str(s),
            OwnedTerm::List(elements) => {
                let mut seq = serializer.serialize_seq(Some(elements.len()))?;
                for elem in elements {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            OwnedTerm::Tuple(elements) => {
                let mut seq = serializer.serialize_seq(Some(elements.len()))?;
                for elem in elements {
                    seq.serialize_element(elem)?;
                }
                seq.end()
            }
            OwnedTerm::Map(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            OwnedTerm::Nil => {
                let seq = serializer.serialize_seq(Some(0))?;
                seq.end()
            }
            OwnedTerm::BigInt(big) => {
                if big.digits.len() <= 8 {
                    let mut bytes = [0u8; 8];
                    bytes[..big.digits.len()].copy_from_slice(&big.digits);
                    let val = u64::from_le_bytes(bytes);
                    if big.sign.is_negative() {
                        if val <= i64::MAX as u64 {
                            serializer.serialize_i64(-(val as i64))
                        } else {
                            serializer.serialize_bytes(&big.digits)
                        }
                    } else {
                        serializer.serialize_u64(val)
                    }
                } else {
                    serializer.serialize_bytes(&big.digits)
                }
            }
            OwnedTerm::ImproperList { elements, tail } => {
                let mut seq = serializer.serialize_seq(Some(elements.len() + 1))?;
                for elem in elements {
                    seq.serialize_element(elem)?;
                }
                seq.serialize_element(tail.as_ref())?;
                seq.end()
            }
            OwnedTerm::BitBinary { bytes, .. } => serializer.serialize_bytes(bytes),
            OwnedTerm::Pid(pid) => serializer.serialize_str(&pid.to_string()),
            OwnedTerm::Port(port) => serializer.serialize_str(&format!("{:?}", port)),
            OwnedTerm::Reference(r) => serializer.serialize_str(&format!("{:?}", r)),
            OwnedTerm::ExternalFun(f) => serializer.serialize_str(&format!("{:?}", f)),
            OwnedTerm::InternalFun(f) => serializer.serialize_str(&format!("{:?}", f)),
        }
    }
}

#[cfg(feature = "serde")]
struct OwnedTermVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for OwnedTermVisitor {
    type Value = OwnedTerm;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any Erlang term")
    }

    fn visit_bool<E>(self, v: bool) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Atom(Atom::new(if v { "true" } else { "false" })))
    }

    fn visit_i64<E>(self, v: i64) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Integer(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<OwnedTerm, E> {
        if v <= i64::MAX as u64 {
            Ok(OwnedTerm::Integer(v as i64))
        } else {
            let bytes = v.to_le_bytes().to_vec();
            Ok(OwnedTerm::BigInt(BigInt::new(false, bytes)))
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Float(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Binary(v.as_bytes().to_vec()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Binary(v.as_bytes().to_vec()))
    }

    fn visit_string<E>(self, v: String) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Binary(v.into_bytes()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Binary(v.to_vec()))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Binary(v.to_vec()))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<OwnedTerm, E> {
        Ok(OwnedTerm::Binary(v))
    }

    fn visit_none<E>(self) -> Result<OwnedTerm, E> {
        #[cfg(feature = "elixir-interop")]
        {
            Ok(OwnedTerm::Atom(Atom::new("nil")))
        }
        #[cfg(not(feature = "elixir-interop"))]
        {
            Ok(OwnedTerm::Atom(Atom::new("undefined")))
        }
    }

    fn visit_some<D>(self, deserializer: D) -> Result<OwnedTerm, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
    }

    fn visit_unit<E>(self) -> Result<OwnedTerm, E> {
        #[cfg(feature = "elixir-interop")]
        {
            Ok(OwnedTerm::Atom(Atom::new("nil")))
        }
        #[cfg(not(feature = "elixir-interop"))]
        {
            Ok(OwnedTerm::Atom(Atom::new("undefined")))
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<OwnedTerm, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut elements = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(elem) = seq.next_element::<OwnedTerm>()? {
            elements.push(elem);
        }
        Ok(OwnedTerm::List(elements))
    }

    fn visit_map<A>(self, mut map: A) -> Result<OwnedTerm, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut m = BTreeMap::new();
        while let Some((k, v)) = map.next_entry::<OwnedTerm, OwnedTerm>()? {
            m.insert(k, v);
        }
        Ok(OwnedTerm::Map(m))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for OwnedTerm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(OwnedTermVisitor)
    }
}
