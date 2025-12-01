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

use crate::borrowed::BorrowedTerm;
use crate::errors::{ContextualDecodeError, DecodeError, ParsingContext, PathSegment};
use crate::tags::{
    ATOM_CACHE_REF, ATOM_EXT, ATOM_UTF8_EXT, BINARY_EXT, BIT_BINARY_EXT, COMPRESSED_EXT,
    DIST_FRAG_HEADER, DIST_HEADER, EXPORT_EXT, FLOAT_EXT, INTEGER_EXT, LARGE_BIG_EXT,
    LARGE_TUPLE_EXT, LIST_EXT, LOCAL_EXT, MAP_EXT, NEW_FLOAT_EXT, NEW_FUN_EXT, NEW_PID_EXT,
    NEW_REFERENCE_EXT, NEWER_REFERENCE_EXT, NIL_EXT, PID_EXT, PORT_EXT, REFERENCE_EXT,
    SMALL_ATOM_EXT, SMALL_ATOM_UTF8_EXT, SMALL_BIG_EXT, SMALL_INTEGER_EXT, SMALL_TUPLE_EXT,
    STRING_EXT, V4_PORT_EXT, VERSION,
};
use crate::term::OwnedTerm;
use crate::types::{
    Atom, BigInt, ExternalFun, ExternalPid, ExternalPort, ExternalReference, InternalFun,
};
use flate2::read::ZlibDecoder;
use nom::IResult;
use nom::bytes::complete::take;
use nom::error::{Error as NomError, ErrorKind};
use nom::number::complete::{be_f64, be_i32, be_u8, be_u16, be_u32, be_u64};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;
use std::str;

const MAX_ATOM_SIZE: usize = 65535;
const MAX_LIST_SIZE: usize = 10_000_000;
const MAX_TUPLE_SIZE: usize = 10_000_000;
const MAX_MAP_SIZE: usize = 1_000_000;
const MAX_BINARY_SIZE: usize = 100_000_000;

type NomResult<'a, T> = IResult<&'a [u8], T, NomError<&'a [u8]>>;

const ATOM_CACHE_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct AtomCache {
    atoms: HashMap<u8, Atom>,
}

impl AtomCache {
    pub fn new() -> Self {
        Self {
            atoms: HashMap::with_capacity(ATOM_CACHE_SIZE),
        }
    }

    pub fn insert(&mut self, index: u8, atom: Atom) {
        self.atoms.insert(index, atom);
    }

    pub fn get(&self, index: u8) -> Option<&Atom> {
        self.atoms.get(&index)
    }

    pub fn len(&self) -> usize {
        self.atoms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }
}

impl Default for AtomCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn decode(data: &[u8]) -> Result<OwnedTerm, DecodeError> {
    let cache = AtomCache::new();
    let (remaining, term) = parse_versioned_term(data, &cache).map_err(from_nom_error)?;

    if !remaining.is_empty() {
        return Err(DecodeError::TrailingData(remaining.len()));
    }

    Ok(term)
}

pub fn decode_with_trailing(data: &[u8]) -> Result<(OwnedTerm, &[u8]), DecodeError> {
    let cache = AtomCache::new();
    let (remaining, term) = parse_versioned_term(data, &cache).map_err(from_nom_error)?;
    Ok((term, remaining))
}

pub fn decode_raw_term(data: &[u8]) -> Result<OwnedTerm, DecodeError> {
    let cache = AtomCache::new();
    let (remaining, term) = parse_term(data, &cache).map_err(from_nom_error)?;

    if !remaining.is_empty() {
        return Err(DecodeError::TrailingData(remaining.len()));
    }

    Ok(term)
}

#[allow(clippy::type_complexity)]
pub fn decode_with_cache(
    data: &[u8],
) -> Result<(OwnedTerm, Option<(OwnedTerm, &[u8])>), DecodeError> {
    let mut cache = AtomCache::new();
    let (remaining, term) =
        parse_versioned_term_with_cache(data, &mut cache).map_err(from_nom_error)?;

    if !remaining.is_empty() {
        let (new_remaining, payload) = parse_term(remaining, &cache).map_err(from_nom_error)?;
        Ok((term, Some((payload, new_remaining))))
    } else {
        Ok((term, None))
    }
}

pub fn decode_with_atom_cache(
    data: &[u8],
    cache: &mut AtomCache,
) -> Result<(OwnedTerm, Option<OwnedTerm>), DecodeError> {
    let (remaining, term) = parse_versioned_term_with_cache(data, cache).map_err(from_nom_error)?;

    if !remaining.is_empty() {
        let (new_remaining, payload) = parse_term(remaining, cache).map_err(from_nom_error)?;
        if !new_remaining.is_empty() {
            return Err(DecodeError::TrailingData(new_remaining.len()));
        }
        Ok((term, Some(payload)))
    } else {
        Ok((term, None))
    }
}

#[derive(Debug, Clone)]
pub struct FragmentHeader {
    pub sequence_id: u64,
    pub fragment_id: u64,
    pub num_atom_cache_refs: u8,
}

pub fn decode_fragment_header(data: &[u8]) -> Result<(FragmentHeader, &[u8]), DecodeError> {
    let (input, version) = be_u8(data).map_err(from_nom_error)?;
    if version != VERSION {
        return Err(DecodeError::InvalidVersion {
            expected: VERSION,
            actual: version,
        });
    }

    let (input, tag) = be_u8(input).map_err(from_nom_error)?;
    if tag != DIST_FRAG_HEADER {
        return Err(DecodeError::InvalidFormat(format!(
            "Expected DIST_FRAG_HEADER ({}), got {}",
            DIST_FRAG_HEADER, tag
        )));
    }

    let (input, sequence_id) = be_u64(input).map_err(from_nom_error)?;
    let (input, fragment_id) = be_u64(input).map_err(from_nom_error)?;
    let (input, num_atom_cache_refs) = be_u8(input).map_err(from_nom_error)?;

    Ok((
        FragmentHeader {
            sequence_id,
            fragment_id,
            num_atom_cache_refs,
        },
        input,
    ))
}

pub fn decode_fragment_cont(data: &[u8]) -> Result<((u64, u64), &[u8]), DecodeError> {
    let (input, version) = be_u8(data).map_err(from_nom_error)?;
    if version != VERSION {
        return Err(DecodeError::InvalidVersion {
            expected: VERSION,
            actual: version,
        });
    }

    let (input, tag) = be_u8(input).map_err(from_nom_error)?;
    if tag != NEW_FLOAT_EXT {
        return Err(DecodeError::InvalidFormat(format!(
            "Expected DIST_FRAG_CONT (70), got {}",
            tag
        )));
    }

    let (input, sequence_id) = be_u64(input).map_err(from_nom_error)?;
    let (input, fragment_id) = be_u64(input).map_err(from_nom_error)?;

    Ok(((sequence_id, fragment_id), input))
}

fn from_nom_error(e: nom::Err<NomError<&[u8]>>) -> DecodeError {
    match e {
        nom::Err::Incomplete(_) => DecodeError::UnexpectedEof,
        nom::Err::Error(e) | nom::Err::Failure(e) => match e.code {
            ErrorKind::Tag => DecodeError::InvalidVersion {
                expected: VERSION,
                actual: 0,
            },
            ErrorKind::Eof => DecodeError::UnexpectedEof,
            ErrorKind::Verify => DecodeError::InvalidFormat("validation failed".to_string()),
            ErrorKind::TooLarge => DecodeError::InvalidFormat("size limit exceeded".to_string()),
            _ => DecodeError::InvalidFormat(format!("{:?}", e.code)),
        },
    }
}

fn parse_versioned_term<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, version) = be_u8(input)?;
    if version != VERSION {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)));
    }
    parse_term(input, cache)
}

fn parse_versioned_term_with_cache<'a>(
    input: &'a [u8],
    cache: &mut AtomCache,
) -> NomResult<'a, OwnedTerm> {
    let (input, version) = be_u8(input)?;
    if version != VERSION {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)));
    }

    let (input, tag) = be_u8(input)?;
    if tag == DIST_HEADER {
        parse_dist_header_with_cache(input, cache)
    } else {
        parse_term_from_tag(input, tag, cache)
    }
}

fn parse_term<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, tag) = be_u8(input)?;
    parse_term_from_tag(input, tag, cache)
}

fn parse_term_from_tag<'a>(
    input: &'a [u8],
    tag: u8,
    cache: &AtomCache,
) -> NomResult<'a, OwnedTerm> {
    match tag {
        SMALL_INTEGER_EXT => parse_small_integer(input),
        INTEGER_EXT => parse_integer(input),
        FLOAT_EXT => parse_old_float(input),
        NEW_FLOAT_EXT => parse_new_float(input),
        ATOM_EXT => parse_atom_latin1(input),
        ATOM_UTF8_EXT => parse_atom_utf8(input),
        SMALL_ATOM_UTF8_EXT => parse_small_atom_utf8(input),
        SMALL_ATOM_EXT => parse_small_atom_latin1(input),
        SMALL_TUPLE_EXT => parse_small_tuple(input, cache),
        LARGE_TUPLE_EXT => parse_large_tuple(input, cache),
        NIL_EXT => Ok((input, OwnedTerm::Nil)),
        STRING_EXT => parse_string_ext(input),
        LIST_EXT => parse_list(input, cache),
        BINARY_EXT => parse_binary(input),
        BIT_BINARY_EXT => parse_bit_binary(input),
        SMALL_BIG_EXT => parse_small_big(input),
        LARGE_BIG_EXT => parse_large_big(input),
        MAP_EXT => parse_map(input, cache),
        NEW_PID_EXT => parse_new_pid(input, cache),
        NEWER_REFERENCE_EXT => parse_newer_reference(input, cache),
        V4_PORT_EXT => parse_v4_port(input, cache),
        EXPORT_EXT => parse_export_ext(input, cache),
        NEW_FUN_EXT => parse_new_fun_ext(input, cache),
        DIST_HEADER => {
            log::error!("DIST_HEADER should not appear nested in terms");
            Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)))
        }
        COMPRESSED_EXT => parse_compressed(input, cache),
        REFERENCE_EXT => parse_reference_ext(input, cache),
        PORT_EXT => parse_port_ext(input, cache),
        PID_EXT => parse_pid_ext(input, cache),
        NEW_REFERENCE_EXT => parse_new_reference_ext(input, cache),
        LOCAL_EXT => parse_local_ext(input, cache),
        ATOM_CACHE_REF => {
            let (input, cache_index) = be_u8(input)?;
            if let Some(atom) = cache.get(cache_index) {
                log::debug!(
                    "Found ATOM_CACHE_REF index {} -> '{}'",
                    cache_index,
                    atom.as_str()
                );
                Ok((input, OwnedTerm::Atom(atom.clone())))
            } else {
                log::error!(
                    "ATOM_CACHE_REF index {} not found in cache (cache size: {})",
                    cache_index,
                    cache.len()
                );
                Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)))
            }
        }
        _ => {
            log::error!("Unknown term tag: {} (0x{:02x})", tag, tag);
            Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)))
        }
    }
}

fn parse_compressed<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (rest, uncompressed_size) = be_u32(input)?;

    if uncompressed_size as usize > MAX_BINARY_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }

    let mut decoder = ZlibDecoder::new(rest);
    let mut decompressed = Vec::with_capacity(uncompressed_size as usize);
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Fail)))?;
    let consumed = decoder.total_in() as usize;

    let owned_term = match parse_term(&decompressed, cache) {
        Ok((_remaining, term)) => term,
        Err(_) => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Fail))),
    };

    Ok((&rest[consumed..], owned_term))
}

fn parse_reference_ext<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, node_term) = parse_term(input, cache)?;
    let node = if let OwnedTerm::Atom(atom) = node_term {
        atom
    } else {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)));
    };
    let (input, id) = be_u32(input)?;
    let (input, creation) = be_u8(input)?;
    Ok((
        input,
        OwnedTerm::Reference(ExternalReference::new(node, creation as u32, vec![id])),
    ))
}

fn parse_port_ext<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, node_term) = parse_term(input, cache)?;
    let node = if let OwnedTerm::Atom(atom) = node_term {
        atom
    } else {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)));
    };
    let (input, id) = be_u32(input)?;
    let (input, creation) = be_u8(input)?;
    Ok((
        input,
        OwnedTerm::Port(ExternalPort::new(node, id as u64, creation as u32)),
    ))
}

fn parse_pid_ext<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, node_term) = parse_term(input, cache)?;
    let node = if let OwnedTerm::Atom(atom) = node_term {
        atom
    } else {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)));
    };
    let (input, id) = be_u32(input)?;
    let (input, serial) = be_u32(input)?;
    let (input, creation) = be_u8(input)?;
    Ok((
        input,
        OwnedTerm::Pid(ExternalPid::new(node, id, serial, creation as u32)),
    ))
}

fn parse_new_reference_ext<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, len) = be_u16(input)?;
    let (input, node_term) = parse_term(input, cache)?;
    let node = if let OwnedTerm::Atom(atom) = node_term {
        atom
    } else {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)));
    };
    let (input, creation) = be_u8(input)?;
    let mut ids = Vec::with_capacity(len as usize);
    let mut remaining = input;
    for _ in 0..len {
        let (rest, id) = be_u32(remaining)?;
        ids.push(id);
        remaining = rest;
    }
    Ok((
        remaining,
        OwnedTerm::Reference(ExternalReference::new(node, creation as u32, ids)),
    ))
}

fn parse_local_ext<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    // Record the start position to capture the entire LOCAL_EXT encoding
    let start = input;
    let (input, _hash) = be_u64(input)?;
    let (remaining, term) = parse_term(input, cache)?;

    // Calculate how many bytes the nested term consumed
    let nested_len = input.len() - remaining.len();
    // LOCAL_EXT bytes = hash (8) + nested term (tag is added during encoding)
    let local_ext_bytes_len = 8 + nested_len;
    let local_ext_bytes = start[..local_ext_bytes_len].to_vec();

    // Preserve LOCAL_EXT bytes for PIDs, ports, and references for transparent re-encoding
    let result = match term {
        OwnedTerm::Pid(pid) => OwnedTerm::Pid(ExternalPid::with_local_ext_bytes(
            pid.node,
            pid.id,
            pid.serial,
            pid.creation,
            local_ext_bytes,
        )),
        OwnedTerm::Port(port) => OwnedTerm::Port(ExternalPort::with_local_ext_bytes(
            port.node,
            port.id,
            port.creation,
            local_ext_bytes,
        )),
        OwnedTerm::Reference(reference) => {
            OwnedTerm::Reference(ExternalReference::with_local_ext_bytes(
                reference.node,
                reference.creation,
                reference.ids,
                local_ext_bytes,
            ))
        }
        _ => term,
    };

    Ok((remaining, result))
}

fn parse_small_integer(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, value) = be_u8(input)?;
    Ok((input, OwnedTerm::Integer(value as i64)))
}

fn parse_integer(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, value) = be_i32(input)?;
    Ok((input, OwnedTerm::Integer(value as i64)))
}

fn parse_old_float(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, bytes) = take(31usize)(input)?;
    let s = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    let value = s
        .trim_end_matches('\0')
        .parse::<f64>()
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Float)))?;
    Ok((input, OwnedTerm::Float(value)))
}

fn parse_new_float(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, value) = be_f64(input)?;
    Ok((input, OwnedTerm::Float(value)))
}

fn parse_atom_latin1(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, len) = be_u16(input)?;
    if len as usize > MAX_ATOM_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    let name = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    Ok((input, OwnedTerm::Atom(Atom::new(name))))
}

fn parse_atom_utf8(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, len) = be_u16(input)?;
    if len as usize > MAX_ATOM_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    let name = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    Ok((input, OwnedTerm::Atom(Atom::new(name))))
}

fn parse_small_atom_utf8(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, len) = be_u8(input)?;
    if len as usize > MAX_ATOM_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    let name = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    Ok((input, OwnedTerm::Atom(Atom::new(name))))
}

fn parse_small_atom_latin1(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, len) = be_u8(input)?;
    if len as usize > MAX_ATOM_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    let name = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    Ok((input, OwnedTerm::Atom(Atom::new(name))))
}

fn parse_dist_header_with_cache<'a>(
    input: &'a [u8],
    cache: &mut AtomCache,
) -> NomResult<'a, OwnedTerm> {
    let (input, num_atom_cache_refs) = be_u8(input)?;

    if num_atom_cache_refs == 0 {
        return parse_term(input, cache);
    }

    let flags_len = (num_atom_cache_refs as usize) / 2 + 1;
    let (mut input, flags) = take(flags_len)(input)?;

    let long_atoms_flag_byte = flags[flags_len - 1];
    let long_atoms = (long_atoms_flag_byte & 0x01) != 0;

    for i in 0..num_atom_cache_refs {
        let (new_input, internal_segment_index) = be_u8(input)?;
        input = new_input;

        let flag_byte_index = i as usize / 2;
        let flag_nibble = if i % 2 == 0 {
            flags[flag_byte_index] & 0x0F
        } else {
            (flags[flag_byte_index] >> 4) & 0x0F
        };

        let is_new_entry = (flag_nibble & 0x08) != 0;

        if is_new_entry {
            let (new_input, atom_len) = if long_atoms {
                let (new_input, len) = be_u16(input)?;
                (new_input, len as usize)
            } else {
                let (new_input, len) = be_u8(input)?;
                (new_input, len as usize)
            };
            input = new_input;

            let (new_input, atom_text) = take(atom_len)(input)?;
            let atom_str = str::from_utf8(atom_text)
                .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;

            log::debug!(
                "Inserting atom '{}' at cache index {}",
                atom_str,
                internal_segment_index
            );
            cache.insert(internal_segment_index, Atom::new(atom_str));
            input = new_input;
        }
    }

    parse_term(input, cache)
}

fn parse_small_tuple<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, arity) = be_u8(input)?;
    if arity as usize > MAX_TUPLE_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let mut remaining = input;
    let mut elements = Vec::with_capacity(arity as usize);

    for _ in 0..arity {
        let (new_remaining, term) = parse_term(remaining, cache)?;
        elements.push(term);
        remaining = new_remaining;
    }

    Ok((remaining, OwnedTerm::Tuple(elements)))
}

fn parse_large_tuple<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, arity) = be_u32(input)?;
    if arity as usize > MAX_TUPLE_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let mut remaining = input;
    let mut elements = Vec::with_capacity(arity as usize);

    for _ in 0..arity {
        let (new_remaining, term) = parse_term(remaining, cache)?;
        elements.push(term);
        remaining = new_remaining;
    }

    Ok((remaining, OwnedTerm::Tuple(elements)))
}

fn parse_string_ext(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, len) = be_u16(input)?;
    let (input, bytes) = take(len as usize)(input)?;
    let elements: Vec<OwnedTerm> = bytes
        .iter()
        .map(|&b| OwnedTerm::Integer(b as i64))
        .collect();
    Ok((input, OwnedTerm::List(elements)))
}

fn parse_list<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, len) = be_u32(input)?;
    if len as usize > MAX_LIST_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let mut remaining = input;
    let mut elements = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let (new_remaining, term) = parse_term(remaining, cache)?;
        elements.push(term);
        remaining = new_remaining;
    }

    let (remaining, tail) = parse_term(remaining, cache)?;

    if tail == OwnedTerm::Nil {
        Ok((remaining, OwnedTerm::List(elements)))
    } else {
        Ok((
            remaining,
            OwnedTerm::ImproperList {
                elements,
                tail: Box::new(tail),
            },
        ))
    }
}

fn parse_binary(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, len) = be_u32(input)?;
    if len as usize > MAX_BINARY_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, data) = take(len as usize)(input)?;
    Ok((input, OwnedTerm::Binary(data.to_vec())))
}

fn parse_bit_binary(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, len) = be_u32(input)?;
    if len as usize > MAX_BINARY_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bits) = be_u8(input)?;
    if bits == 0 || bits > 8 {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Verify)));
    }
    if len == 0 && bits != 8 {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Verify)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    Ok((
        input,
        OwnedTerm::BitBinary {
            bytes: bytes.to_vec(),
            bits,
        },
    ))
}

fn parse_small_big(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, n) = be_u8(input)?;
    let (input, sign) = be_u8(input)?;
    let (input, digits) = take(n as usize)(input)?;
    Ok((
        input,
        OwnedTerm::BigInt(BigInt::new(sign != 0, digits.to_vec())),
    ))
}

fn parse_large_big(input: &[u8]) -> NomResult<'_, OwnedTerm> {
    let (input, n) = be_u32(input)?;
    let (input, sign) = be_u8(input)?;
    let (input, digits) = take(n as usize)(input)?;
    Ok((
        input,
        OwnedTerm::BigInt(BigInt::new(sign != 0, digits.to_vec())),
    ))
}

fn parse_map<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, arity) = be_u32(input)?;
    if arity as usize > MAX_MAP_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let mut remaining = input;
    let mut map = BTreeMap::new();

    for _ in 0..arity {
        let (new_remaining, key) = parse_term(remaining, cache)?;
        let (new_remaining, value) = parse_term(new_remaining, cache)?;
        map.insert(key, value);
        remaining = new_remaining;
    }

    Ok((remaining, OwnedTerm::Map(map)))
}

fn parse_new_pid<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, node_term) = parse_term(input, cache)?;
    let node = match node_term {
        OwnedTerm::Atom(a) => a,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, id) = be_u32(input)?;
    let (input, serial) = be_u32(input)?;
    let (input, creation) = be_u32(input)?;

    // NEW_PID_EXT doesn't need raw bytes preserved: its fields are sufficient
    // for precisely reconstructing the term.
    Ok((
        input,
        OwnedTerm::Pid(ExternalPid::new(node, id, serial, creation)),
    ))
}

fn parse_newer_reference<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, len) = be_u16(input)?;
    let (input, node_term) = parse_term(input, cache)?;
    let node = match node_term {
        OwnedTerm::Atom(a) => a,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, creation) = be_u32(input)?;

    let mut remaining = input;
    let mut ids = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let (new_remaining, id) = be_u32(remaining)?;
        ids.push(id);
        remaining = new_remaining;
    }

    Ok((
        remaining,
        OwnedTerm::Reference(ExternalReference::new(node, creation, ids)),
    ))
}

fn parse_v4_port<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, node_term) = parse_term(input, cache)?;
    let node = match node_term {
        OwnedTerm::Atom(a) => a,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, id) = be_u64(input)?;
    let (input, creation) = be_u32(input)?;

    Ok((
        input,
        OwnedTerm::Port(ExternalPort::new(node, id, creation)),
    ))
}

fn parse_export_ext<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, module_term) = parse_term(input, cache)?;
    let module = match module_term {
        OwnedTerm::Atom(a) => a,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, function_term) = parse_term(input, cache)?;
    let function = match function_term {
        OwnedTerm::Atom(a) => a,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, arity_term) = parse_term(input, cache)?;
    let arity = match arity_term {
        OwnedTerm::Integer(i) if (0..=255).contains(&i) => i as u8,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    Ok((
        input,
        OwnedTerm::ExternalFun(ExternalFun::new(module, function, arity)),
    ))
}

fn parse_new_fun_ext<'a>(input: &'a [u8], cache: &AtomCache) -> NomResult<'a, OwnedTerm> {
    let (input, _size) = be_u32(input)?;
    let (input, arity) = be_u8(input)?;
    let (input, uniq) = take(16usize)(input)?;
    let (input, index) = be_u32(input)?;
    let (input, num_free) = be_u32(input)?;

    let (input, module_term) = parse_term(input, cache)?;
    let module = match module_term {
        OwnedTerm::Atom(a) => a,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, old_index_term) = parse_term(input, cache)?;
    let old_index = match old_index_term {
        OwnedTerm::Integer(i) if i >= 0 => i as u32,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, old_uniq_term) = parse_term(input, cache)?;
    let old_uniq = match old_uniq_term {
        OwnedTerm::Integer(i) if i >= 0 => i as u32,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, pid_term) = parse_term(input, cache)?;
    let pid = match pid_term {
        OwnedTerm::Pid(p) => p,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let mut remaining = input;
    let mut free_vars = Vec::with_capacity(num_free as usize);
    for _ in 0..num_free {
        let (new_remaining, term) = parse_term(remaining, cache)?;
        free_vars.push(term);
        remaining = new_remaining;
    }

    let mut uniq_array = [0u8; 16];
    uniq_array.copy_from_slice(uniq);

    Ok((
        remaining,
        OwnedTerm::InternalFun(Box::new(InternalFun::new(
            arity, uniq_array, index, num_free, module, old_index, old_uniq, pid, free_vars,
        ))),
    ))
}

pub fn decode_borrowed(data: &[u8]) -> Result<BorrowedTerm<'_>, ContextualDecodeError> {
    let original_len = data.len();
    let mut ctx = ParsingContext::new();

    let (remaining, term) = parse_versioned_term_borrowed(data, original_len, &mut ctx)
        .map_err(|e| ContextualDecodeError::new(from_nom_error(e), ctx.clone()))?;

    if !remaining.is_empty() {
        ctx.byte_offset = original_len - remaining.len();
        return Err(ContextualDecodeError::new(
            DecodeError::TrailingData(remaining.len()),
            ctx,
        ));
    }

    Ok(term)
}

fn parse_versioned_term_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, version) = be_u8(input)?;
    ctx.byte_offset = original_len - input.len() - 1;
    if version != VERSION {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag)));
    }
    parse_term_borrowed(input, original_len, ctx)
}

fn parse_term_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    ctx.byte_offset = original_len - input.len();
    let (input, tag) = be_u8(input)?;

    match tag {
        SMALL_INTEGER_EXT => parse_small_integer_borrowed(input),
        INTEGER_EXT => parse_integer_borrowed(input),
        FLOAT_EXT => parse_old_float_borrowed(input),
        NEW_FLOAT_EXT => parse_new_float_borrowed(input),
        ATOM_EXT => parse_atom_latin1_borrowed(input),
        ATOM_UTF8_EXT => parse_atom_utf8_borrowed(input),
        SMALL_ATOM_UTF8_EXT => parse_small_atom_utf8_borrowed(input),
        SMALL_TUPLE_EXT => parse_small_tuple_borrowed(input, original_len, ctx),
        LARGE_TUPLE_EXT => parse_large_tuple_borrowed(input, original_len, ctx),
        NIL_EXT => Ok((input, BorrowedTerm::Nil)),
        STRING_EXT => parse_string_ext_borrowed(input),
        LIST_EXT => parse_list_borrowed(input, original_len, ctx),
        BINARY_EXT => parse_binary_borrowed(input),
        BIT_BINARY_EXT => parse_bit_binary_borrowed(input),
        SMALL_BIG_EXT => parse_small_big_borrowed(input),
        LARGE_BIG_EXT => parse_large_big_borrowed(input),
        MAP_EXT => parse_map_borrowed(input, original_len, ctx),
        NEW_PID_EXT => parse_new_pid_borrowed(input, original_len, ctx),
        NEWER_REFERENCE_EXT => parse_newer_reference_borrowed(input, original_len, ctx),
        V4_PORT_EXT => parse_v4_port_borrowed(input, original_len, ctx),
        EXPORT_EXT => parse_export_ext_borrowed(input, original_len, ctx),
        NEW_FUN_EXT => parse_new_fun_ext_borrowed(input, original_len, ctx),
        _ => Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    }
}

fn parse_small_integer_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, value) = be_u8(input)?;
    Ok((input, BorrowedTerm::Integer(value as i64)))
}

fn parse_integer_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, value) = be_i32(input)?;
    Ok((input, BorrowedTerm::Integer(value as i64)))
}

fn parse_old_float_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, bytes) = take(31usize)(input)?;
    let s = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    let value = s
        .trim_end_matches('\0')
        .parse::<f64>()
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Float)))?;
    Ok((input, BorrowedTerm::Float(value)))
}

fn parse_new_float_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, value) = be_f64(input)?;
    Ok((input, BorrowedTerm::Float(value)))
}

fn parse_atom_latin1_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, len) = be_u16(input)?;
    if len as usize > MAX_ATOM_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    let name = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    Ok((input, BorrowedTerm::Atom(Cow::Borrowed(name))))
}

fn parse_atom_utf8_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, len) = be_u16(input)?;
    if len as usize > MAX_ATOM_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    let name = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    Ok((input, BorrowedTerm::Atom(Cow::Borrowed(name))))
}

fn parse_small_atom_utf8_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, len) = be_u8(input)?;
    if len as usize > MAX_ATOM_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    let name = str::from_utf8(bytes)
        .map_err(|_| nom::Err::Failure(NomError::new(input, ErrorKind::Char)))?;
    Ok((input, BorrowedTerm::Atom(Cow::Borrowed(name))))
}

fn parse_small_tuple_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, arity) = be_u8(input)?;
    if arity as usize > MAX_TUPLE_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let mut remaining = input;
    let mut elements = Vec::with_capacity(arity as usize);

    for i in 0..arity {
        ctx.push(PathSegment::TupleElement(i as usize));
        let (new_remaining, term) = parse_term_borrowed(remaining, original_len, ctx)?;
        ctx.pop();
        elements.push(term);
        remaining = new_remaining;
    }

    Ok((remaining, BorrowedTerm::Tuple(elements)))
}

fn parse_large_tuple_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, arity) = be_u32(input)?;
    if arity as usize > MAX_TUPLE_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let mut remaining = input;
    let mut elements = Vec::with_capacity(arity as usize);

    for i in 0..arity {
        ctx.push(PathSegment::TupleElement(i as usize));
        let (new_remaining, term) = parse_term_borrowed(remaining, original_len, ctx)?;
        ctx.pop();
        elements.push(term);
        remaining = new_remaining;
    }

    Ok((remaining, BorrowedTerm::Tuple(elements)))
}

fn parse_string_ext_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, len) = be_u16(input)?;
    let (input, bytes) = take(len as usize)(input)?;
    let elements: Vec<BorrowedTerm<'_>> = bytes
        .iter()
        .map(|&b| BorrowedTerm::Integer(b as i64))
        .collect();
    Ok((input, BorrowedTerm::List(elements)))
}

fn parse_list_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, len) = be_u32(input)?;
    if len as usize > MAX_LIST_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let mut remaining = input;
    let mut elements = Vec::with_capacity(len as usize);

    for i in 0..len {
        ctx.push(PathSegment::ListElement(i as usize));
        let (new_remaining, term) = parse_term_borrowed(remaining, original_len, ctx)?;
        ctx.pop();
        elements.push(term);
        remaining = new_remaining;
    }

    ctx.push(PathSegment::ImproperListTail);
    let (remaining, tail) = parse_term_borrowed(remaining, original_len, ctx)?;
    ctx.pop();

    if tail == BorrowedTerm::Nil {
        Ok((remaining, BorrowedTerm::List(elements)))
    } else {
        Ok((
            remaining,
            BorrowedTerm::ImproperList {
                elements,
                tail: Box::new(tail),
            },
        ))
    }
}

fn parse_binary_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, len) = be_u32(input)?;
    if len as usize > MAX_BINARY_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, data) = take(len as usize)(input)?;
    Ok((input, BorrowedTerm::Binary(Cow::Borrowed(data))))
}

fn parse_bit_binary_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, len) = be_u32(input)?;
    if len as usize > MAX_BINARY_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let (input, bits) = be_u8(input)?;
    if bits == 0 || bits > 8 {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Verify)));
    }
    if len == 0 && bits != 8 {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Verify)));
    }
    let (input, bytes) = take(len as usize)(input)?;
    Ok((
        input,
        BorrowedTerm::BitBinary {
            bytes: Cow::Borrowed(bytes),
            bits,
        },
    ))
}

fn parse_small_big_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, n) = be_u8(input)?;
    let (input, sign) = be_u8(input)?;
    let (input, digits) = take(n as usize)(input)?;
    Ok((
        input,
        BorrowedTerm::BigInt(BigInt::new(sign != 0, digits.to_vec())),
    ))
}

fn parse_large_big_borrowed(input: &[u8]) -> NomResult<'_, BorrowedTerm<'_>> {
    let (input, n) = be_u32(input)?;
    let (input, sign) = be_u8(input)?;
    let (input, digits) = take(n as usize)(input)?;
    Ok((
        input,
        BorrowedTerm::BigInt(BigInt::new(sign != 0, digits.to_vec())),
    ))
}

fn parse_map_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, arity) = be_u32(input)?;
    if arity as usize > MAX_MAP_SIZE {
        return Err(nom::Err::Failure(NomError::new(input, ErrorKind::TooLarge)));
    }
    let mut remaining = input;
    let mut map = BTreeMap::new();

    for _ in 0..arity {
        ctx.push(PathSegment::MapKey);
        let (new_remaining, key) = parse_term_borrowed(remaining, original_len, ctx)?;
        ctx.pop();

        let key_display = match &key {
            BorrowedTerm::Atom(a) => a.to_string(),
            BorrowedTerm::Integer(i) => i.to_string(),
            _ => "?".to_string(),
        };
        ctx.push(PathSegment::MapValue(key_display));
        let (new_remaining, value) = parse_term_borrowed(new_remaining, original_len, ctx)?;
        ctx.pop();

        map.insert(key, value);
        remaining = new_remaining;
    }

    Ok((remaining, BorrowedTerm::Map(map)))
}

fn parse_new_pid_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, node_term) = parse_term_borrowed(input, original_len, ctx)?;
    let node = match node_term {
        BorrowedTerm::Atom(a) => Atom::new(a.as_ref()),
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, id) = be_u32(input)?;
    let (input, serial) = be_u32(input)?;
    let (input, creation) = be_u32(input)?;

    Ok((
        input,
        BorrowedTerm::Pid(ExternalPid::new(node, id, serial, creation)),
    ))
}

fn parse_newer_reference_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, len) = be_u16(input)?;
    let (input, node_term) = parse_term_borrowed(input, original_len, ctx)?;
    let node = match node_term {
        BorrowedTerm::Atom(a) => Atom::new(a.as_ref()),
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, creation) = be_u32(input)?;

    let mut remaining = input;
    let mut ids = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let (new_remaining, id) = be_u32(remaining)?;
        ids.push(id);
        remaining = new_remaining;
    }

    Ok((
        remaining,
        BorrowedTerm::Reference(ExternalReference::new(node, creation, ids)),
    ))
}

fn parse_v4_port_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, node_term) = parse_term_borrowed(input, original_len, ctx)?;
    let node = match node_term {
        BorrowedTerm::Atom(a) => Atom::new(a.as_ref()),
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, id) = be_u64(input)?;
    let (input, creation) = be_u32(input)?;

    Ok((
        input,
        BorrowedTerm::Port(ExternalPort::new(node, id, creation)),
    ))
}

fn parse_export_ext_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, module_term) = parse_term_borrowed(input, original_len, ctx)?;
    let module = match module_term {
        BorrowedTerm::Atom(a) => Atom::new(a.as_ref()),
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, function_term) = parse_term_borrowed(input, original_len, ctx)?;
    let function = match function_term {
        BorrowedTerm::Atom(a) => Atom::new(a.as_ref()),
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, arity_term) = parse_term_borrowed(input, original_len, ctx)?;
    let arity = match arity_term {
        BorrowedTerm::Integer(i) if (0..=255).contains(&i) => i as u8,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    Ok((
        input,
        BorrowedTerm::ExternalFun(ExternalFun::new(module, function, arity)),
    ))
}

fn parse_new_fun_ext_borrowed<'a>(
    input: &'a [u8],
    original_len: usize,
    ctx: &mut ParsingContext,
) -> NomResult<'a, BorrowedTerm<'a>> {
    let (input, _size) = be_u32(input)?;
    let (input, arity) = be_u8(input)?;
    let (input, uniq) = take(16usize)(input)?;
    let (input, index) = be_u32(input)?;
    let (input, num_free) = be_u32(input)?;

    let (input, module_term) = parse_term_borrowed(input, original_len, ctx)?;
    let module = match module_term {
        BorrowedTerm::Atom(a) => Atom::new(a.as_ref()),
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, old_index_term) = parse_term_borrowed(input, original_len, ctx)?;
    let old_index = match old_index_term {
        BorrowedTerm::Integer(i) if i >= 0 => i as u32,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, old_uniq_term) = parse_term_borrowed(input, original_len, ctx)?;
    let old_uniq = match old_uniq_term {
        BorrowedTerm::Integer(i) if i >= 0 => i as u32,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let (input, pid_term) = parse_term_borrowed(input, original_len, ctx)?;
    let pid = match pid_term {
        BorrowedTerm::Pid(p) => p,
        _ => return Err(nom::Err::Failure(NomError::new(input, ErrorKind::Tag))),
    };

    let mut remaining = input;
    let mut free_vars = Vec::with_capacity(num_free as usize);
    for i in 0..num_free {
        ctx.push(PathSegment::FunFreeVar(i as usize));
        let (new_remaining, term) = parse_term_borrowed(remaining, original_len, ctx)?;
        ctx.pop();
        free_vars.push(term.to_owned());
        remaining = new_remaining;
    }

    let mut uniq_array = [0u8; 16];
    uniq_array.copy_from_slice(uniq);

    Ok((
        remaining,
        BorrowedTerm::InternalFun(Box::new(InternalFun::new(
            arity, uniq_array, index, num_free, module, old_index, old_uniq, pid, free_vars,
        ))),
    ))
}
