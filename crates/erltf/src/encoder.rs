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

use crate::errors::EncodeError;
use crate::tags::{
    ATOM_CACHE_REF, ATOM_UTF8_EXT, BINARY_EXT, BIT_BINARY_EXT, DIST_HEADER, EXPORT_EXT,
    INTEGER_EXT, LARGE_BIG_EXT, LARGE_TUPLE_EXT, LIST_EXT, LOCAL_EXT, MAP_EXT, NEW_FLOAT_EXT,
    NEW_FUN_EXT, NEW_PID_EXT, NEWER_REFERENCE_EXT, NIL_EXT, SMALL_ATOM_UTF8_EXT, SMALL_BIG_EXT,
    SMALL_INTEGER_EXT, SMALL_TUPLE_EXT, V4_PORT_EXT, VERSION,
};
use crate::term::OwnedTerm;
use crate::types::{
    Atom, BigInt, ExternalFun, ExternalPid, ExternalPort, ExternalReference, InternalFun,
};
use bytes::{BufMut, BytesMut};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;

pub fn encode(term: &OwnedTerm) -> Result<Vec<u8>, EncodeError> {
    let estimated_size = term.estimated_encoded_size() + 1;
    let capacity = estimated_size.max(64);
    let mut buf = BytesMut::with_capacity(capacity);
    buf.put_u8(VERSION);
    encode_term(&mut buf, term)?;
    Ok(buf.to_vec())
}

pub fn encode_to_writer<W: Write>(term: &OwnedTerm, writer: &mut W) -> Result<(), EncodeError> {
    let encoded = encode(term)?;
    writer.write_all(&encoded)?;
    Ok(())
}

fn encode_term(buf: &mut BytesMut, term: &OwnedTerm) -> Result<(), EncodeError> {
    encode_term_impl(buf, term, None)
}

fn encode_term_impl<'a>(
    buf: &mut BytesMut,
    term: &'a OwnedTerm,
    cache: Option<&HashMap<&'a Atom, u8>>,
) -> Result<(), EncodeError> {
    match term {
        OwnedTerm::Atom(atom) => encode_atom_impl(buf, atom, cache),
        OwnedTerm::Integer(i) => encode_integer(buf, *i),
        OwnedTerm::Float(f) => encode_float(buf, *f),
        OwnedTerm::Binary(b) => encode_binary(buf, b),
        OwnedTerm::BitBinary { bytes, bits } => encode_bit_binary(buf, bytes, *bits),
        OwnedTerm::String(s) => encode_string(buf, s),
        OwnedTerm::List(l) => encode_list_impl(buf, l, cache),
        OwnedTerm::ImproperList { elements, tail } => {
            encode_improper_list_impl(buf, elements, tail, cache)
        }
        OwnedTerm::Map(m) => encode_map_impl(buf, m, cache),
        OwnedTerm::Tuple(t) => encode_tuple_impl(buf, t, cache),
        OwnedTerm::Pid(pid) => encode_pid_impl(buf, pid, cache),
        OwnedTerm::Port(port) => encode_port_impl(buf, port, cache),
        OwnedTerm::Reference(ref_) => encode_reference_impl(buf, ref_, cache),
        OwnedTerm::BigInt(big) => encode_bigint(buf, big),
        OwnedTerm::ExternalFun(fun) => encode_export_ext_impl(buf, fun, cache),
        OwnedTerm::InternalFun(fun) => encode_new_fun_ext_impl(buf, fun, cache),
        OwnedTerm::Nil => encode_nil(buf),
    }
}

fn encode_atom_impl<'a>(
    buf: &mut BytesMut,
    atom: &'a Atom,
    cache: Option<&HashMap<&'a Atom, u8>>,
) -> Result<(), EncodeError> {
    if let Some(atom_index_map) = cache
        && let Some(&cache_index) = atom_index_map.get(&atom)
    {
        buf.put_u8(ATOM_CACHE_REF);
        buf.put_u8(cache_index);
        return Ok(());
    }

    let bytes = atom.name.as_bytes();
    let len = bytes.len();

    if len > u16::MAX as usize {
        return Err(EncodeError::AtomTooLarge { size: len });
    }

    if len > 255 {
        buf.put_u8(ATOM_UTF8_EXT);
        buf.put_u16(len as u16);
    } else {
        buf.put_u8(SMALL_ATOM_UTF8_EXT);
        buf.put_u8(len as u8);
    }
    buf.put_slice(bytes);
    Ok(())
}

fn encode_integer(buf: &mut BytesMut, value: i64) -> Result<(), EncodeError> {
    if (0..=255).contains(&value) {
        buf.put_u8(SMALL_INTEGER_EXT);
        buf.put_u8(value as u8);
    } else if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
        buf.put_u8(INTEGER_EXT);
        buf.put_i32(value as i32);
    } else {
        let (sign, abs_value) = if value >= 0 {
            (0u8, value as u64)
        } else {
            (1u8, value.wrapping_neg() as u64)
        };

        let le_bytes = abs_value.to_le_bytes();
        let significant_len = le_bytes
            .iter()
            .rposition(|&b| b != 0)
            .map_or(1, |pos| pos + 1);

        if significant_len <= 255 {
            buf.put_u8(SMALL_BIG_EXT);
            buf.put_u8(significant_len as u8);
            buf.put_u8(sign);
            buf.put_slice(&le_bytes[..significant_len]);
        } else {
            buf.put_u8(LARGE_BIG_EXT);
            buf.put_u32(significant_len as u32);
            buf.put_u8(sign);
            buf.put_slice(&le_bytes[..significant_len]);
        }
    }
    Ok(())
}

fn encode_float(buf: &mut BytesMut, value: f64) -> Result<(), EncodeError> {
    buf.put_u8(NEW_FLOAT_EXT);
    buf.put_f64(value);
    Ok(())
}

fn encode_binary(buf: &mut BytesMut, data: &[u8]) -> Result<(), EncodeError> {
    let len =
        u32::try_from(data.len()).map_err(|_| EncodeError::BinaryTooLarge { size: data.len() })?;
    buf.put_u8(BINARY_EXT);
    buf.put_u32(len);
    buf.put_slice(data);
    Ok(())
}

fn encode_bit_binary(buf: &mut BytesMut, bytes: &[u8], bits: u8) -> Result<(), EncodeError> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| EncodeError::BinaryTooLarge { size: bytes.len() })?;
    buf.put_u8(BIT_BINARY_EXT);
    buf.put_u32(len);
    buf.put_u8(bits);
    buf.put_slice(bytes);
    Ok(())
}

fn encode_string(buf: &mut BytesMut, s: &str) -> Result<(), EncodeError> {
    encode_binary(buf, s.as_bytes())
}

fn encode_list_impl<'a>(
    buf: &mut BytesMut,
    elements: &'a [OwnedTerm],
    cache: Option<&HashMap<&'a Atom, u8>>,
) -> Result<(), EncodeError> {
    if elements.is_empty() {
        return encode_nil(buf);
    }

    let len = u32::try_from(elements.len()).map_err(|_| EncodeError::ListTooLarge {
        size: elements.len(),
    })?;

    buf.put_u8(LIST_EXT);
    buf.put_u32(len);
    for elem in elements {
        encode_term_impl(buf, elem, cache)?;
    }
    encode_nil(buf)?;
    Ok(())
}

fn encode_improper_list_impl<'a>(
    buf: &mut BytesMut,
    elements: &'a [OwnedTerm],
    tail: &'a OwnedTerm,
    cache: Option<&HashMap<&'a Atom, u8>>,
) -> Result<(), EncodeError> {
    let len = u32::try_from(elements.len()).map_err(|_| EncodeError::ListTooLarge {
        size: elements.len(),
    })?;

    buf.put_u8(LIST_EXT);
    buf.put_u32(len);
    for elem in elements {
        encode_term_impl(buf, elem, cache)?;
    }
    encode_term_impl(buf, tail, cache)?;
    Ok(())
}

fn encode_map_impl<'a>(
    buf: &mut BytesMut,
    map: &'a BTreeMap<OwnedTerm, OwnedTerm>,
    cache: Option<&HashMap<&'a Atom, u8>>,
) -> Result<(), EncodeError> {
    let len = u32::try_from(map.len()).map_err(|_| EncodeError::MapTooLarge { size: map.len() })?;

    buf.put_u8(MAP_EXT);
    buf.put_u32(len);

    for (key, value) in map.iter() {
        encode_term_impl(buf, key, cache)?;
        encode_term_impl(buf, value, cache)?;
    }
    Ok(())
}

fn encode_tuple_impl(
    buf: &mut BytesMut,
    elements: &[OwnedTerm],
    cache: Option<&HashMap<&Atom, u8>>,
) -> Result<(), EncodeError> {
    if elements.len() <= 255 {
        buf.put_u8(SMALL_TUPLE_EXT);
        buf.put_u8(elements.len() as u8);
    } else {
        let len = u32::try_from(elements.len()).map_err(|_| EncodeError::TupleTooLarge {
            size: elements.len(),
        })?;
        buf.put_u8(LARGE_TUPLE_EXT);
        buf.put_u32(len);
    }
    for elem in elements {
        encode_term_impl(buf, elem, cache)?;
    }
    Ok(())
}

fn encode_pid_impl(
    buf: &mut BytesMut,
    pid: &ExternalPid,
    cache: Option<&HashMap<&Atom, u8>>,
) -> Result<(), EncodeError> {
    // If this PID was decoded from LOCAL_EXT, use the preserved bytes for transparent re-encoding.
    // Otherwise, encode as NEW_PID_EXT (which can be exactly reconstructed from parsed fields).
    if let Some(local_bytes) = &pid.local_ext_bytes {
        buf.put_u8(LOCAL_EXT);
        buf.put_slice(local_bytes);
    } else {
        buf.put_u8(NEW_PID_EXT);
        encode_atom_impl(buf, &pid.node, cache)?;
        buf.put_u32(pid.id);
        buf.put_u32(pid.serial);
        buf.put_u32(pid.creation);
    }
    Ok(())
}

fn encode_port_impl(
    buf: &mut BytesMut,
    port: &ExternalPort,
    cache: Option<&HashMap<&Atom, u8>>,
) -> Result<(), EncodeError> {
    // Use preserved LOCAL_EXT bytes if available for transparent re-encoding
    if let Some(ref local_ext_bytes) = port.local_ext_bytes {
        buf.put_u8(LOCAL_EXT);
        buf.put_slice(local_ext_bytes);
    } else {
        buf.put_u8(V4_PORT_EXT);
        encode_atom_impl(buf, &port.node, cache)?;
        buf.put_u64(port.id);
        buf.put_u32(port.creation);
    }
    Ok(())
}

fn encode_reference_impl(
    buf: &mut BytesMut,
    ref_: &ExternalReference,
    cache: Option<&HashMap<&Atom, u8>>,
) -> Result<(), EncodeError> {
    // Use preserved LOCAL_EXT bytes if available for transparent re-encoding
    if let Some(ref local_ext_bytes) = ref_.local_ext_bytes {
        buf.put_u8(LOCAL_EXT);
        buf.put_slice(local_ext_bytes);
    } else {
        let len = u16::try_from(ref_.ids.len()).map_err(|_| EncodeError::ReferenceTooLarge {
            size: ref_.ids.len(),
        })?;

        buf.put_u8(NEWER_REFERENCE_EXT);
        buf.put_u16(len);
        encode_atom_impl(buf, &ref_.node, cache)?;
        buf.put_u32(ref_.creation);
        for id in &ref_.ids {
            buf.put_u32(*id);
        }
    }
    Ok(())
}

fn encode_bigint(buf: &mut BytesMut, big: &BigInt) -> Result<(), EncodeError> {
    let len = big.digits.len();
    if len <= 255 {
        buf.put_u8(SMALL_BIG_EXT);
        buf.put_u8(len as u8);
    } else {
        buf.put_u8(LARGE_BIG_EXT);
        buf.put_u32(len as u32);
    }
    buf.put_u8(if big.sign.is_negative() { 1 } else { 0 });
    buf.put_slice(&big.digits);
    Ok(())
}

fn encode_nil(buf: &mut BytesMut) -> Result<(), EncodeError> {
    buf.put_u8(NIL_EXT);
    Ok(())
}

fn encode_export_ext_impl(
    buf: &mut BytesMut,
    fun: &ExternalFun,
    cache: Option<&HashMap<&Atom, u8>>,
) -> Result<(), EncodeError> {
    buf.put_u8(EXPORT_EXT);
    encode_atom_impl(buf, &fun.module, cache)?;
    encode_atom_impl(buf, &fun.function, cache)?;
    encode_integer(buf, fun.arity as i64)?;
    Ok(())
}

fn encode_new_fun_ext_impl(
    buf: &mut BytesMut,
    fun: &InternalFun,
    cache: Option<&HashMap<&Atom, u8>>,
) -> Result<(), EncodeError> {
    let mut temp_buf = BytesMut::new();

    temp_buf.put_u8(fun.arity);
    temp_buf.put_slice(&fun.uniq);
    temp_buf.put_u32(fun.index);
    temp_buf.put_u32(fun.num_free);

    encode_atom_impl(&mut temp_buf, &fun.module, cache)?;
    encode_integer(&mut temp_buf, fun.old_index as i64)?;
    encode_integer(&mut temp_buf, fun.old_uniq as i64)?;
    encode_pid_impl(&mut temp_buf, &fun.pid, cache)?;

    for var in &fun.free_vars {
        encode_term_impl(&mut temp_buf, var, cache)?;
    }

    buf.put_u8(NEW_FUN_EXT);
    buf.put_u32((temp_buf.len() + 4) as u32);
    buf.put_slice(&temp_buf);

    Ok(())
}

fn collect_atoms<'a>(term: &'a OwnedTerm, atoms: &mut HashSet<&'a Atom>) {
    match term {
        OwnedTerm::Atom(atom) => {
            atoms.insert(atom);
        }
        OwnedTerm::Tuple(elements) | OwnedTerm::List(elements) => {
            for elem in elements {
                collect_atoms(elem, atoms);
            }
        }
        OwnedTerm::ImproperList { elements, tail } => {
            for elem in elements {
                collect_atoms(elem, atoms);
            }
            collect_atoms(tail, atoms);
        }
        OwnedTerm::Map(map) => {
            for (key, value) in map {
                collect_atoms(key, atoms);
                collect_atoms(value, atoms);
            }
        }
        OwnedTerm::Pid(pid) => {
            atoms.insert(&pid.node);
        }
        OwnedTerm::Port(port) => {
            atoms.insert(&port.node);
        }
        OwnedTerm::Reference(ref_) => {
            atoms.insert(&ref_.node);
        }
        OwnedTerm::ExternalFun(fun) => {
            atoms.insert(&fun.module);
            atoms.insert(&fun.function);
        }
        OwnedTerm::InternalFun(fun) => {
            atoms.insert(&fun.module);
            atoms.insert(&fun.pid.node);
            for var in &fun.free_vars {
                collect_atoms(var, atoms);
            }
        }
        _ => {}
    }
}

fn encode_term_with_cache<'a>(
    buf: &mut BytesMut,
    term: &'a OwnedTerm,
    atom_index_map: &HashMap<&'a Atom, u8>,
) -> Result<(), EncodeError> {
    encode_term_impl(buf, term, Some(atom_index_map))
}

pub fn encode_with_dist_header(term: &OwnedTerm) -> Result<Vec<u8>, EncodeError> {
    encode_with_dist_header_multi(&[term])
}

pub fn encode_with_dist_header_multi(terms: &[&OwnedTerm]) -> Result<Vec<u8>, EncodeError> {
    let mut atom_set = HashSet::new();
    for term in terms {
        collect_atoms(term, &mut atom_set);
    }

    if atom_set.is_empty() {
        let mut buf = BytesMut::new();
        buf.put_u8(VERSION);
        for term in terms {
            encode_term(&mut buf, term)?;
        }
        return Ok(buf.to_vec());
    }

    if atom_set.len() > 255 {
        return Err(EncodeError::TooManyAtoms {
            count: atom_set.len(),
        });
    }

    let atoms: Vec<&Atom> = atom_set.iter().copied().collect();

    let mut atom_index_map = HashMap::new();
    for (index, atom) in atoms.iter().enumerate() {
        atom_index_map.insert(*atom, index as u8);
    }

    let estimated_size = terms
        .iter()
        .map(|t| t.estimated_encoded_size())
        .sum::<usize>()
        + atoms.len() * 10
        + 64;
    let mut buf = BytesMut::with_capacity(estimated_size);

    buf.put_u8(VERSION);
    buf.put_u8(DIST_HEADER);
    buf.put_u8(atoms.len() as u8);

    let flags_len = (atoms.len() / 2) + 1;
    let flags_start_pos = buf.len();
    for _ in 0..flags_len {
        buf.put_u8(0);
    }

    let long_atoms = atoms.iter().any(|a| a.name.len() > 255);
    if long_atoms {
        buf[flags_start_pos + flags_len - 1] |= 0x01;
    }

    for (index, atom) in atoms.iter().enumerate() {
        buf.put_u8(index as u8);

        let flag_byte_index = index / 2;
        let nibble_shift = if index % 2 == 0 { 0 } else { 4 };
        let new_entry_flag = 0x08u8;

        let flag_pos = flags_start_pos + flag_byte_index;
        buf[flag_pos] |= new_entry_flag << nibble_shift;

        let atom_bytes = atom.name.as_bytes();
        let atom_len = atom_bytes.len();

        if long_atoms {
            buf.put_u16(atom_len as u16);
        } else {
            buf.put_u8(atom_len as u8);
        }

        buf.put_slice(atom_bytes);
    }

    for term in terms {
        encode_term_with_cache(&mut buf, term, &atom_index_map)?;
    }

    Ok(buf.to_vec())
}
