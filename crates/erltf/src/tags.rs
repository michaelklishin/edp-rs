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

//! External Term Format (ETF) tag constants.
//!
//! This module contains all the tag byte values used in Erlang's External Term Format.
//! See: <https://www.erlang.org/doc/apps/erts/erl_ext_dist>

/// ETF version byte (always 131)
pub const VERSION: u8 = 131;

// Atom tags
pub const ATOM_EXT: u8 = 100;
pub const SMALL_ATOM_EXT: u8 = 115;
pub const ATOM_UTF8_EXT: u8 = 118;
pub const SMALL_ATOM_UTF8_EXT: u8 = 119;
pub const ATOM_CACHE_REF: u8 = 82;

// Integer tags
pub const SMALL_INTEGER_EXT: u8 = 97;
pub const INTEGER_EXT: u8 = 98;
pub const SMALL_BIG_EXT: u8 = 110;
pub const LARGE_BIG_EXT: u8 = 111;

// Float tags
pub const FLOAT_EXT: u8 = 99;
pub const NEW_FLOAT_EXT: u8 = 70;

// Container tags
pub const SMALL_TUPLE_EXT: u8 = 104;
pub const LARGE_TUPLE_EXT: u8 = 105;
pub const NIL_EXT: u8 = 106;
pub const STRING_EXT: u8 = 107;
pub const LIST_EXT: u8 = 108;
pub const MAP_EXT: u8 = 116;

// Binary tags
pub const BINARY_EXT: u8 = 109;
pub const BIT_BINARY_EXT: u8 = 77;

// Process/Port/Reference tags (legacy)
pub const REFERENCE_EXT: u8 = 101;
pub const PORT_EXT: u8 = 102;
pub const PID_EXT: u8 = 103;
pub const NEW_REFERENCE_EXT: u8 = 114;

// Process/Port/Reference tags (modern)
pub const NEW_PID_EXT: u8 = 88;
pub const NEWER_REFERENCE_EXT: u8 = 90;
pub const V4_PORT_EXT: u8 = 120;

// Local-only encoding (OTP 26+)
pub const LOCAL_EXT: u8 = 121;

// Function tags
pub const NEW_FUN_EXT: u8 = 112;
pub const EXPORT_EXT: u8 = 113;

// Distribution header tags
pub const DIST_HEADER: u8 = 68;
pub const DIST_FRAG_HEADER: u8 = 69;

// Compression
pub const COMPRESSED_EXT: u8 = 80;
