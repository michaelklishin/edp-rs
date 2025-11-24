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

use crate::error::{Error, Result};
use erltf::term::OwnedTerm;
use erltf::types::Atom;
use serde::de::{DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::{Deserialize, Deserializer as SerdeDeserializer};
use std::collections::BTreeMap;
use std::collections::btree_map;

pub fn from_bytes<T: for<'a> Deserialize<'a>>(bytes: &[u8]) -> Result<T> {
    let term = erltf::decode(bytes).map_err(|e| Error::Erltf(e.into()))?;
    from_term(&term)
}

pub fn from_term<'a, T: Deserialize<'a>>(term: &'a OwnedTerm) -> Result<T> {
    let mut deserializer = Deserializer { term };
    T::deserialize(&mut deserializer)
}

pub struct Deserializer<'de> {
    term: &'de OwnedTerm,
}

impl<'de> Deserializer<'de> {
    fn expect_atom(&self, expected: &str) -> Result<&Atom> {
        match self.term {
            OwnedTerm::Atom(atom) => {
                if atom.as_str() == expected {
                    Ok(atom)
                } else {
                    Err(Error::TypeMismatch {
                        expected: format!("atom '{}'", expected),
                        found: format!("atom '{}'", atom.as_str()),
                    })
                }
            }
            _ => Err(Error::TypeMismatch {
                expected: format!("atom '{}'", expected),
                found: format!("{:?}", self.term),
            }),
        }
    }
}

impl<'de> SerdeDeserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Atom(atom) => match atom.as_str() {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                "nil" => visitor.visit_unit(),
                "undefined" => visitor.visit_none(),
                _ => visitor.visit_str(atom.as_str()),
            },
            OwnedTerm::Integer(i) => visitor.visit_i64(*i),
            OwnedTerm::Float(f) => visitor.visit_f64(*f),
            OwnedTerm::Binary(b) => {
                if let Ok(s) = std::str::from_utf8(b) {
                    visitor.visit_str(s)
                } else {
                    visitor.visit_bytes(b)
                }
            }
            OwnedTerm::String(s) => visitor.visit_str(s),
            OwnedTerm::List(l) => visitor.visit_seq(SeqDeserializer::new(l)),
            OwnedTerm::Tuple(t) => visitor.visit_seq(SeqDeserializer::new(t)),
            OwnedTerm::Map(m) => visitor.visit_map(MapDeserializer::new(m)),
            OwnedTerm::Nil => visitor.visit_seq(SeqDeserializer::new(&[])),
            _ => Err(Error::UnsupportedType(format!("{:?}", self.term))),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Atom(atom) => match atom.as_str() {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                _ => Err(Error::TypeMismatch {
                    expected: "bool atom".into(),
                    found: format!("{:?}", atom),
                }),
            },
            _ => Err(Error::TypeMismatch {
                expected: "bool atom".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Integer(i) => i8::try_from(*i)
                .map_err(|_| Error::InvalidValue(format!("integer {} out of range for i8", i)))
                .and_then(|v| visitor.visit_i8(v)),
            _ => Err(Error::TypeMismatch {
                expected: "integer".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Integer(i) => i16::try_from(*i)
                .map_err(|_| Error::InvalidValue(format!("integer {} out of range for i16", i)))
                .and_then(|v| visitor.visit_i16(v)),
            _ => Err(Error::TypeMismatch {
                expected: "integer".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Integer(i) => i32::try_from(*i)
                .map_err(|_| Error::InvalidValue(format!("integer {} out of range for i32", i)))
                .and_then(|v| visitor.visit_i32(v)),
            _ => Err(Error::TypeMismatch {
                expected: "integer".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Integer(i) => visitor.visit_i64(*i),
            _ => Err(Error::TypeMismatch {
                expected: "integer".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Integer(i) => u8::try_from(*i)
                .map_err(|_| Error::InvalidValue(format!("integer {} out of range for u8", i)))
                .and_then(|v| visitor.visit_u8(v)),
            _ => Err(Error::TypeMismatch {
                expected: "integer".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Integer(i) => u16::try_from(*i)
                .map_err(|_| Error::InvalidValue(format!("integer {} out of range for u16", i)))
                .and_then(|v| visitor.visit_u16(v)),
            _ => Err(Error::TypeMismatch {
                expected: "integer".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Integer(i) => u32::try_from(*i)
                .map_err(|_| Error::InvalidValue(format!("integer {} out of range for u32", i)))
                .and_then(|v| visitor.visit_u32(v)),
            _ => Err(Error::TypeMismatch {
                expected: "integer".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Integer(i) => u64::try_from(*i)
                .map_err(|_| Error::InvalidValue(format!("integer {} out of range for u64", i)))
                .and_then(|v| visitor.visit_u64(v)),
            OwnedTerm::BigInt(big) if big.sign.is_positive() && big.digits.len() <= 8 => {
                let mut bytes = [0u8; 8];
                bytes[..big.digits.len()].copy_from_slice(&big.digits);
                let value = u64::from_le_bytes(bytes);
                visitor.visit_u64(value)
            }
            _ => Err(Error::TypeMismatch {
                expected: "integer or unsigned bigint".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Float(f) => visitor.visit_f32(*f as f32),
            _ => Err(Error::TypeMismatch {
                expected: "float".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Float(f) => visitor.visit_f64(*f),
            _ => Err(Error::TypeMismatch {
                expected: "float".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::String(s) => {
                let mut chars = s.chars();
                if let Some(c) = chars.next()
                    && chars.next().is_none()
                {
                    return visitor.visit_char(c);
                }
                Err(Error::InvalidValue("expected single char".into()))
            }
            _ => Err(Error::TypeMismatch {
                expected: "string".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Binary(b) => {
                let s = std::str::from_utf8(b).map_err(|e| Error::InvalidValue(e.to_string()))?;
                visitor.visit_borrowed_str(s)
            }
            OwnedTerm::String(s) => visitor.visit_borrowed_str(s),
            OwnedTerm::Atom(a) => visitor.visit_borrowed_str(a.as_str()),
            _ => Err(Error::TypeMismatch {
                expected: "string or binary".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Binary(b) => visitor.visit_borrowed_bytes(b),
            _ => Err(Error::TypeMismatch {
                expected: "binary".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Atom(atom) if atom.as_str() == "undefined" => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.expect_atom("nil")?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.expect_atom(name)?;
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::List(l) => visitor.visit_seq(SeqDeserializer::new(l)),
            OwnedTerm::Nil => visitor.visit_seq(SeqDeserializer::new(&[])),
            _ => Err(Error::TypeMismatch {
                expected: "list".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Tuple(t) => visitor.visit_seq(SeqDeserializer::new(t)),
            _ => Err(Error::TypeMismatch {
                expected: "tuple".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Tuple(t) => visitor.visit_seq(SeqDeserializer::new(t)),
            _ => Err(Error::TypeMismatch {
                expected: "tuple".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Map(m) => visitor.visit_map(MapDeserializer::new(m)),
            _ => Err(Error::TypeMismatch {
                expected: "map".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        match self.term {
            OwnedTerm::Atom(_) => visitor.visit_enum(EnumDeserializer { term: self.term }),
            OwnedTerm::Tuple(elements) if !elements.is_empty() => {
                visitor.visit_enum(EnumDeserializer { term: self.term })
            }
            _ => Err(Error::TypeMismatch {
                expected: "atom or tuple".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }
}

struct SeqDeserializer<'de> {
    iter: std::slice::Iter<'de, OwnedTerm>,
}

impl<'de> SeqDeserializer<'de> {
    fn new(slice: &'de [OwnedTerm]) -> Self {
        SeqDeserializer { iter: slice.iter() }
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        match self.iter.next() {
            Some(term) => {
                let mut de = Deserializer { term };
                seed.deserialize(&mut de).map(Some)
            }
            None => Ok(None),
        }
    }
}

struct MapDeserializer<'de> {
    iter: btree_map::Iter<'de, OwnedTerm, OwnedTerm>,
    value: Option<&'de OwnedTerm>,
}

impl<'de> MapDeserializer<'de> {
    fn new(map: &'de BTreeMap<OwnedTerm, OwnedTerm>) -> Self {
        MapDeserializer {
            iter: map.iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let mut de = Deserializer { term: key };
                seed.deserialize(&mut de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        match self.value.take() {
            Some(value) => {
                let mut de = Deserializer { term: value };
                seed.deserialize(&mut de)
            }
            None => Err(Error::Message("next_value called without next_key".into())),
        }
    }
}

struct EnumDeserializer<'de> {
    term: &'de OwnedTerm,
}

impl<'de> EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;
    type Variant = VariantDeserializer<'de>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        match self.term {
            OwnedTerm::Atom(_) => {
                let mut de = Deserializer { term: self.term };
                let val = seed.deserialize(&mut de)?;
                Ok((val, VariantDeserializer { rest: &[] }))
            }
            OwnedTerm::Tuple(elements) if !elements.is_empty() => {
                let mut de = Deserializer { term: &elements[0] };
                let val = seed.deserialize(&mut de)?;
                let rest = if elements.len() > 1 {
                    &elements[1..]
                } else {
                    &elements[0..0]
                };
                Ok((val, VariantDeserializer { rest }))
            }
            _ => Err(Error::TypeMismatch {
                expected: "enum (atom or tuple)".into(),
                found: format!("{:?}", self.term),
            }),
        }
    }
}

struct VariantDeserializer<'de> {
    rest: &'de [OwnedTerm],
}

impl<'de> VariantAccess<'de> for VariantDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(Error::TypeMismatch {
                expected: "unit variant".into(),
                found: format!("variant with {} elements", self.rest.len()),
            })
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        if self.rest.len() == 1 {
            let mut de = Deserializer {
                term: &self.rest[0],
            };
            seed.deserialize(&mut de)
        } else {
            Err(Error::TypeMismatch {
                expected: "newtype variant with 1 element".into(),
                found: format!("variant with {} elements", self.rest.len()),
            })
        }
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(SeqDeserializer::new(self.rest))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        if self.rest.len() == 1 {
            match &self.rest[0] {
                OwnedTerm::Map(m) => visitor.visit_map(MapDeserializer::new(m)),
                _ => Err(Error::TypeMismatch {
                    expected: "struct variant (map)".into(),
                    found: format!("{:?}", self.rest[0]),
                }),
            }
        } else {
            Err(Error::TypeMismatch {
                expected: "struct variant with 1 map element".into(),
                found: format!("variant with {} elements", self.rest.len()),
            })
        }
    }
}
