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
use erltf::types::{Atom, BigInt};
use serde::Serializer as SerdeSerializer;
use serde::ser::{self, Serialize};
use std::collections::BTreeMap;

pub fn to_term<T: Serialize>(value: &T) -> Result<OwnedTerm> {
    let mut serializer = Serializer;
    value.serialize(&mut serializer)
}

pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let term = to_term(value)?;
    erltf::encode(&term).map_err(|e| Error::Erltf(e.into()))
}

pub struct Serializer;

impl SerdeSerializer for &mut Serializer {
    type Ok = OwnedTerm;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Atom(Atom::new(if v { "true" } else { "false" })))
    }

    fn serialize_i8(self, v: i8) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Integer(v as i64))
    }

    fn serialize_i16(self, v: i16) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Integer(v as i64))
    }

    fn serialize_i32(self, v: i32) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Integer(v as i64))
    }

    fn serialize_i64(self, v: i64) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Integer(v as i64))
    }

    fn serialize_u16(self, v: u16) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Integer(v as i64))
    }

    fn serialize_u32(self, v: u32) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Integer(v as i64))
    }

    fn serialize_u64(self, v: u64) -> Result<OwnedTerm> {
        if v <= i64::MAX as u64 {
            Ok(OwnedTerm::Integer(v as i64))
        } else {
            let le_bytes = v.to_le_bytes();
            let digits = le_bytes.to_vec();
            Ok(OwnedTerm::BigInt(BigInt::new(false, digits)))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<OwnedTerm> {
        Ok(OwnedTerm::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Binary(v.as_bytes().to_vec()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Binary(v.to_vec()))
    }

    fn serialize_none(self) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Atom(Atom::new("undefined")))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<OwnedTerm> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Atom(Atom::new("nil")))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Atom(Atom::new(name)))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Atom(Atom::new(variant)))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<OwnedTerm> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<OwnedTerm> {
        let val = value.serialize(&mut Serializer)?;
        Ok(OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new(variant)),
            val,
        ]))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeVec { vec: Vec::new() })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(SerializeVec { vec: Vec::new() })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(SerializeVec { vec: Vec::new() })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            name: variant,
            vec: Vec::new(),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap {
            map: BTreeMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeMap {
            map: BTreeMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeStructVariant {
            name: variant,
            map: BTreeMap::new(),
        })
    }
}

pub struct SerializeVec {
    vec: Vec<OwnedTerm>,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = OwnedTerm;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.vec.push(value.serialize(&mut Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<OwnedTerm> {
        Ok(OwnedTerm::List(self.vec))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = OwnedTerm;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.vec.push(value.serialize(&mut Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Tuple(self.vec))
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = OwnedTerm;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.vec.push(value.serialize(&mut Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Tuple(self.vec))
    }
}

pub struct SerializeTupleVariant {
    name: &'static str,
    vec: Vec<OwnedTerm>,
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = OwnedTerm;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.vec.push(value.serialize(&mut Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<OwnedTerm> {
        let mut elements = vec![OwnedTerm::Atom(Atom::new(self.name))];
        elements.extend(self.vec);
        Ok(OwnedTerm::Tuple(elements))
    }
}

pub struct SerializeMap {
    map: BTreeMap<OwnedTerm, OwnedTerm>,
    next_key: Option<OwnedTerm>,
}

impl ser::SerializeMap for SerializeMap {
    type Ok = OwnedTerm;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.next_key = Some(key.serialize(&mut Serializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| Error::Message("serialize_value called without serialize_key".into()))?;
        self.map.insert(key, value.serialize(&mut Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Map(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = OwnedTerm;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        let key_term = OwnedTerm::Binary(key.as_bytes().to_vec());
        self.map.insert(key_term, value.serialize(&mut Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Map(self.map))
    }
}

pub struct SerializeStructVariant {
    name: &'static str,
    map: BTreeMap<OwnedTerm, OwnedTerm>,
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = OwnedTerm;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        let key_term = OwnedTerm::Binary(key.as_bytes().to_vec());
        self.map.insert(key_term, value.serialize(&mut Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<OwnedTerm> {
        Ok(OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new(self.name)),
            OwnedTerm::Map(self.map),
        ]))
    }
}
