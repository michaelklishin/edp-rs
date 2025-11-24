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

use crate::term::OwnedTerm;
use crate::types::{
    Atom, BigInt, ExternalFun, ExternalPid, ExternalPort, ExternalReference, InternalFun, Sign,
};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
pub enum BorrowedTerm<'a> {
    Atom(Cow<'a, str>),
    Integer(i64),
    Float(f64),
    Pid(ExternalPid),
    Port(ExternalPort),
    Reference(ExternalReference),
    Binary(Cow<'a, [u8]>),
    BitBinary {
        bytes: Cow<'a, [u8]>,
        bits: u8,
    },
    String(Cow<'a, str>),
    List(Vec<BorrowedTerm<'a>>),
    ImproperList {
        elements: Vec<BorrowedTerm<'a>>,
        tail: Box<BorrowedTerm<'a>>,
    },
    Map(BTreeMap<BorrowedTerm<'a>, BorrowedTerm<'a>>),
    Tuple(Vec<BorrowedTerm<'a>>),
    BigInt(BigInt),
    ExternalFun(ExternalFun),
    InternalFun(Box<InternalFun>),
    Nil,
}

impl<'a> BorrowedTerm<'a> {
    pub fn to_owned(&self) -> OwnedTerm {
        match self {
            BorrowedTerm::Atom(s) => OwnedTerm::Atom(Atom::new(s.as_ref())),
            BorrowedTerm::Integer(i) => OwnedTerm::Integer(*i),
            BorrowedTerm::Float(f) => OwnedTerm::Float(*f),
            BorrowedTerm::Pid(p) => OwnedTerm::Pid(p.clone()),
            BorrowedTerm::Port(p) => OwnedTerm::Port(p.clone()),
            BorrowedTerm::Reference(r) => OwnedTerm::Reference(r.clone()),
            BorrowedTerm::Binary(b) => OwnedTerm::Binary(b.as_ref().to_vec()),
            BorrowedTerm::BitBinary { bytes, bits } => OwnedTerm::BitBinary {
                bytes: bytes.as_ref().to_vec(),
                bits: *bits,
            },
            BorrowedTerm::String(s) => OwnedTerm::String(s.to_string()),
            BorrowedTerm::List(elements) => {
                let mut owned = Vec::with_capacity(elements.len());
                for e in elements {
                    owned.push(e.to_owned());
                }
                OwnedTerm::List(owned)
            }
            BorrowedTerm::ImproperList { elements, tail } => {
                let mut owned = Vec::with_capacity(elements.len());
                for e in elements {
                    owned.push(e.to_owned());
                }
                OwnedTerm::ImproperList {
                    elements: owned,
                    tail: Box::new(tail.as_ref().to_owned()),
                }
            }
            BorrowedTerm::Map(m) => OwnedTerm::Map(
                m.iter()
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    .collect(),
            ),
            BorrowedTerm::Tuple(elements) => {
                let mut owned = Vec::with_capacity(elements.len());
                for e in elements {
                    owned.push(e.to_owned());
                }
                OwnedTerm::Tuple(owned)
            }
            BorrowedTerm::BigInt(b) => OwnedTerm::BigInt(b.clone()),
            BorrowedTerm::ExternalFun(f) => OwnedTerm::ExternalFun(f.clone()),
            BorrowedTerm::InternalFun(f) => OwnedTerm::InternalFun(f.clone()),
            BorrowedTerm::Nil => OwnedTerm::Nil,
        }
    }

    #[inline]
    pub fn is_borrowed(&self) -> bool {
        match self {
            BorrowedTerm::Atom(s) => matches!(s, Cow::Borrowed(_)),
            BorrowedTerm::Binary(b) => matches!(b, Cow::Borrowed(_)),
            BorrowedTerm::BitBinary { bytes, .. } => matches!(bytes, Cow::Borrowed(_)),
            BorrowedTerm::String(s) => matches!(s, Cow::Borrowed(_)),
            BorrowedTerm::List(elements) => elements.iter().any(|e| e.is_borrowed()),
            BorrowedTerm::ImproperList { elements, tail } => {
                elements.iter().any(|e| e.is_borrowed()) || tail.is_borrowed()
            }
            BorrowedTerm::Map(m) => m.iter().any(|(k, v)| k.is_borrowed() || v.is_borrowed()),
            BorrowedTerm::Tuple(elements) => elements.iter().any(|e| e.is_borrowed()),
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_atom(&self) -> bool {
        matches!(self, BorrowedTerm::Atom(_))
    }

    #[inline]
    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(self, BorrowedTerm::Integer(_))
    }

    #[inline]
    #[must_use]
    pub fn is_list(&self) -> bool {
        matches!(self, BorrowedTerm::List(_) | BorrowedTerm::Nil)
    }

    #[inline]
    #[must_use]
    pub fn is_map(&self) -> bool {
        matches!(self, BorrowedTerm::Map(_))
    }

    #[inline]
    #[must_use]
    pub fn is_tuple(&self) -> bool {
        matches!(self, BorrowedTerm::Tuple(_))
    }

    #[inline]
    #[must_use]
    pub fn as_atom(&self) -> Option<&str> {
        match self {
            BorrowedTerm::Atom(a) => Some(a.as_ref()),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            BorrowedTerm::Integer(i) => Some(*i),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            BorrowedTerm::Float(f) => Some(*f),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            BorrowedTerm::Binary(b) => Some(b.as_ref()),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            BorrowedTerm::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_list(&self) -> Option<&[BorrowedTerm<'a>]> {
        match self {
            BorrowedTerm::List(l) => Some(l),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_map(&self) -> Option<&BTreeMap<BorrowedTerm<'a>, BorrowedTerm<'a>>> {
        match self {
            BorrowedTerm::Map(m) => Some(m),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn as_tuple(&self) -> Option<&[BorrowedTerm<'a>]> {
        match self {
            BorrowedTerm::Tuple(t) => Some(t),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            BorrowedTerm::Atom(_) => "Atom",
            BorrowedTerm::Integer(_) => "Integer",
            BorrowedTerm::Float(_) => "Float",
            BorrowedTerm::Pid(_) => "Pid",
            BorrowedTerm::Port(_) => "Port",
            BorrowedTerm::Reference(_) => "Reference",
            BorrowedTerm::Binary(_) => "Binary",
            BorrowedTerm::BitBinary { .. } => "BitBinary",
            BorrowedTerm::String(_) => "String",
            BorrowedTerm::List(_) => "List",
            BorrowedTerm::ImproperList { .. } => "ImproperList",
            BorrowedTerm::Map(_) => "Map",
            BorrowedTerm::Tuple(_) => "Tuple",
            BorrowedTerm::BigInt(_) => "BigInt",
            BorrowedTerm::ExternalFun(_) => "ExternalFun",
            BorrowedTerm::InternalFun(_) => "InternalFun",
            BorrowedTerm::Nil => "Nil",
        }
    }

    #[inline]
    #[must_use]
    pub fn atom_name(&self) -> Option<&str> {
        match self {
            BorrowedTerm::Atom(a) => Some(a.as_ref()),
            _ => None,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_atom_with_name(&self, name: &str) -> bool {
        match self {
            BorrowedTerm::Atom(a) => a.as_ref() == name,
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
    pub fn as_bool(&self) -> Option<bool> {
        self.atom_name().and_then(|name| match name {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            BorrowedTerm::List(l) => l.len(),
            BorrowedTerm::Tuple(t) => t.len(),
            BorrowedTerm::Map(m) => m.len(),
            BorrowedTerm::Binary(b) => b.len(),
            BorrowedTerm::String(s) => s.len(),
            BorrowedTerm::Nil => 0,
            _ => 0,
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            BorrowedTerm::List(l) => l.is_empty(),
            BorrowedTerm::Tuple(t) => t.is_empty(),
            BorrowedTerm::Map(m) => m.is_empty(),
            BorrowedTerm::Binary(b) => b.is_empty(),
            BorrowedTerm::String(s) => s.is_empty(),
            BorrowedTerm::Nil => true,
            _ => false,
        }
    }

    pub fn map_get(&self, key: &BorrowedTerm<'a>) -> Option<&BorrowedTerm<'a>> {
        match self {
            BorrowedTerm::Map(m) => m.get(key),
            _ => None,
        }
    }

    pub fn iter(&self) -> BorrowedTermIter<'_> {
        match self {
            BorrowedTerm::List(elements) | BorrowedTerm::Tuple(elements) => {
                BorrowedTermIter::Slice(elements.iter())
            }
            BorrowedTerm::Nil => BorrowedTermIter::Empty,
            _ => BorrowedTermIter::Empty,
        }
    }

    pub fn map_iter(
        &self,
    ) -> Option<impl Iterator<Item = (&BorrowedTerm<'a>, &BorrowedTerm<'a>)> + '_> {
        match self {
            BorrowedTerm::Map(m) => Some(m.iter()),
            _ => None,
        }
    }
}

pub enum BorrowedTermIter<'a> {
    Slice(std::slice::Iter<'a, BorrowedTerm<'a>>),
    Empty,
}

impl<'a> Iterator for BorrowedTermIter<'a> {
    type Item = &'a BorrowedTerm<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BorrowedTermIter::Slice(iter) => iter.next(),
            BorrowedTermIter::Empty => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BorrowedTermIter::Slice(iter) => iter.size_hint(),
            BorrowedTermIter::Empty => (0, Some(0)),
        }
    }
}

impl<'a> ExactSizeIterator for BorrowedTermIter<'a> {
    fn len(&self) -> usize {
        match self {
            BorrowedTermIter::Slice(iter) => iter.len(),
            BorrowedTermIter::Empty => 0,
        }
    }
}

impl<'a> From<&'a OwnedTerm> for BorrowedTerm<'a> {
    fn from(term: &'a OwnedTerm) -> Self {
        match term {
            OwnedTerm::Atom(a) => BorrowedTerm::Atom(Cow::Borrowed(a.as_str())),
            OwnedTerm::Integer(i) => BorrowedTerm::Integer(*i),
            OwnedTerm::Float(f) => BorrowedTerm::Float(*f),
            OwnedTerm::Pid(p) => BorrowedTerm::Pid(p.clone()),
            OwnedTerm::Port(p) => BorrowedTerm::Port(p.clone()),
            OwnedTerm::Reference(r) => BorrowedTerm::Reference(r.clone()),
            OwnedTerm::Binary(b) => BorrowedTerm::Binary(Cow::Borrowed(b.as_slice())),
            OwnedTerm::BitBinary { bytes, bits } => BorrowedTerm::BitBinary {
                bytes: Cow::Borrowed(bytes.as_slice()),
                bits: *bits,
            },
            OwnedTerm::String(s) => BorrowedTerm::String(Cow::Borrowed(s.as_str())),
            OwnedTerm::List(elements) => {
                BorrowedTerm::List(elements.iter().map(BorrowedTerm::from).collect())
            }
            OwnedTerm::ImproperList { elements, tail } => BorrowedTerm::ImproperList {
                elements: elements.iter().map(BorrowedTerm::from).collect(),
                tail: Box::new(BorrowedTerm::from(tail.as_ref())),
            },
            OwnedTerm::Map(m) => BorrowedTerm::Map(
                m.iter()
                    .map(|(k, v)| (BorrowedTerm::from(k), BorrowedTerm::from(v)))
                    .collect(),
            ),
            OwnedTerm::Tuple(elements) => {
                BorrowedTerm::Tuple(elements.iter().map(BorrowedTerm::from).collect())
            }
            OwnedTerm::BigInt(b) => BorrowedTerm::BigInt(b.clone()),
            OwnedTerm::ExternalFun(f) => BorrowedTerm::ExternalFun(f.clone()),
            OwnedTerm::InternalFun(f) => BorrowedTerm::InternalFun(f.clone()),
            OwnedTerm::Nil => BorrowedTerm::Nil,
        }
    }
}

impl<'a> Eq for BorrowedTerm<'a> {}

impl<'a> Ord for BorrowedTerm<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        let type_order = |t: &BorrowedTerm| -> u8 {
            match t {
                BorrowedTerm::Integer(_) | BorrowedTerm::BigInt(_) | BorrowedTerm::Float(_) => 0,
                BorrowedTerm::Atom(_) => 1,
                BorrowedTerm::Reference(_) => 2,
                BorrowedTerm::ExternalFun(_) | BorrowedTerm::InternalFun(_) => 3,
                BorrowedTerm::Port(_) => 4,
                BorrowedTerm::Pid(_) => 5,
                BorrowedTerm::Tuple(_) => 6,
                BorrowedTerm::Map(_) => 7,
                BorrowedTerm::Nil | BorrowedTerm::List(_) | BorrowedTerm::ImproperList { .. } => 8,
                BorrowedTerm::Binary(_)
                | BorrowedTerm::BitBinary { .. }
                | BorrowedTerm::String(_) => 9,
            }
        };

        match type_order(self).cmp(&type_order(other)) {
            Ordering::Equal => match (self, other) {
                (BorrowedTerm::Integer(a), BorrowedTerm::Integer(b)) => a.cmp(b),
                (BorrowedTerm::Integer(a), BorrowedTerm::BigInt(b)) => compare_int_bigint(*a, b),
                (BorrowedTerm::BigInt(a), BorrowedTerm::Integer(b)) => compare_bigint_int(a, *b),
                (BorrowedTerm::BigInt(a), BorrowedTerm::BigInt(b)) => compare_bigint(a, b),
                (BorrowedTerm::Integer(a), BorrowedTerm::Float(b)) => compare_int_float(*a, *b),
                (BorrowedTerm::Float(a), BorrowedTerm::Integer(b)) => compare_float_int(*a, *b),
                (BorrowedTerm::BigInt(a), BorrowedTerm::Float(b)) => compare_bigint_float(a, *b),
                (BorrowedTerm::Float(a), BorrowedTerm::BigInt(b)) => compare_float_bigint(*a, b),
                (BorrowedTerm::Float(a), BorrowedTerm::Float(b)) => {
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
                (BorrowedTerm::Atom(a), BorrowedTerm::Atom(b)) => a.cmp(b),
                (BorrowedTerm::Reference(a), BorrowedTerm::Reference(b)) => a
                    .node
                    .name
                    .cmp(&b.node.name)
                    .then_with(|| a.creation.cmp(&b.creation))
                    .then_with(|| a.ids.cmp(&b.ids)),
                (BorrowedTerm::ExternalFun(a), BorrowedTerm::ExternalFun(b)) => a
                    .module
                    .name
                    .cmp(&b.module.name)
                    .then_with(|| a.function.name.cmp(&b.function.name))
                    .then_with(|| a.arity.cmp(&b.arity)),
                (BorrowedTerm::InternalFun(a), BorrowedTerm::InternalFun(b)) => a
                    .module
                    .name
                    .cmp(&b.module.name)
                    .then_with(|| a.old_index.cmp(&b.old_index))
                    .then_with(|| a.old_uniq.cmp(&b.old_uniq))
                    .then_with(|| a.index.cmp(&b.index))
                    .then_with(|| a.uniq.cmp(&b.uniq))
                    .then_with(|| a.pid.cmp(&b.pid))
                    .then_with(|| compare_owned_term_lists(&a.free_vars, &b.free_vars)),
                (BorrowedTerm::ExternalFun(_), BorrowedTerm::InternalFun(_)) => Ordering::Less,
                (BorrowedTerm::InternalFun(_), BorrowedTerm::ExternalFun(_)) => Ordering::Greater,
                (BorrowedTerm::Port(a), BorrowedTerm::Port(b)) => a
                    .node
                    .name
                    .cmp(&b.node.name)
                    .then_with(|| a.id.cmp(&b.id))
                    .then_with(|| a.creation.cmp(&b.creation)),
                (BorrowedTerm::Pid(a), BorrowedTerm::Pid(b)) => a
                    .node
                    .name
                    .cmp(&b.node.name)
                    .then_with(|| a.id.cmp(&b.id))
                    .then_with(|| a.serial.cmp(&b.serial))
                    .then_with(|| a.creation.cmp(&b.creation)),
                (BorrowedTerm::Tuple(a), BorrowedTerm::Tuple(b)) => {
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
                (BorrowedTerm::Map(a), BorrowedTerm::Map(b)) => {
                    a.len().cmp(&b.len()).then_with(|| {
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
                    })
                }
                (BorrowedTerm::Nil, BorrowedTerm::Nil) => Ordering::Equal,
                (BorrowedTerm::List(a), BorrowedTerm::List(b)) => {
                    for (x, y) in a.iter().zip(b.iter()) {
                        match x.cmp(y) {
                            Ordering::Equal => continue,
                            other => return other,
                        }
                    }
                    a.len().cmp(&b.len())
                }
                (BorrowedTerm::List(a), BorrowedTerm::Nil) => {
                    if a.is_empty() {
                        Ordering::Equal
                    } else {
                        Ordering::Greater
                    }
                }
                (BorrowedTerm::Nil, BorrowedTerm::List(b)) => {
                    if b.is_empty() {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                }
                (
                    BorrowedTerm::ImproperList {
                        elements: a,
                        tail: ta,
                    },
                    BorrowedTerm::ImproperList {
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
                (BorrowedTerm::Binary(a), BorrowedTerm::Binary(b)) => a.cmp(b),
                (BorrowedTerm::String(a), BorrowedTerm::String(b)) => a.cmp(b),
                (BorrowedTerm::Binary(a), BorrowedTerm::String(b)) => a.as_ref().cmp(b.as_bytes()),
                (BorrowedTerm::String(a), BorrowedTerm::Binary(b)) => a.as_bytes().cmp(b.as_ref()),
                (
                    BorrowedTerm::BitBinary {
                        bytes: a,
                        bits: abits,
                    },
                    BorrowedTerm::BitBinary {
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

impl<'a> PartialOrd for BorrowedTerm<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Index<usize> for BorrowedTerm<'a> {
    type Output = BorrowedTerm<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            BorrowedTerm::List(elements) | BorrowedTerm::Tuple(elements) => &elements[index],
            BorrowedTerm::Nil => panic!(
                "index out of bounds: the len is 0 but the index is {}",
                index
            ),
            _ => panic!("cannot index into {}", self.type_name()),
        }
    }
}

impl<'a> Index<&BorrowedTerm<'a>> for BorrowedTerm<'a> {
    type Output = BorrowedTerm<'a>;

    fn index(&self, key: &BorrowedTerm<'a>) -> &Self::Output {
        match self {
            BorrowedTerm::Map(m) => m.get(key).unwrap_or_else(|| panic!("key not found in map")),
            _ => panic!("cannot index {} with a key", self.type_name()),
        }
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

fn compare_owned_term_lists(a: &[OwnedTerm], b: &[OwnedTerm]) -> Ordering {
    for (x, y) in a.iter().zip(b.iter()) {
        match x.cmp(y) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    a.len().cmp(&b.len())
}
