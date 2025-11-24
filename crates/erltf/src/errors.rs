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

use std::fmt;
use std::result::Result as StdResult;
use std::str::Utf8Error;
use thiserror::Error;

pub type Result<T> = StdResult<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("encode error: {0}")]
    Encode(#[from] EncodeError),
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecodeError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid tag: {0}")]
    InvalidTag(u8),
    #[error("invalid version: expected {expected}, got {actual}")]
    InvalidVersion { expected: u8, actual: u8 },
    #[error("invalid UTF-8 in atom: {0}")]
    InvalidUtf8(String),
    #[error("atom too large: {size} bytes (max {max})")]
    AtomTooLarge { size: usize, max: usize },
    #[error("list too large: {size} elements (max {max})")]
    ListTooLarge { size: usize, max: usize },
    #[error("tuple too large: {size} elements (max {max})")]
    TupleTooLarge { size: usize, max: usize },
    #[error("map too large: {size} entries (max {max})")]
    MapTooLarge { size: usize, max: usize },
    #[error("binary too large: {size} bytes (max {max})")]
    BinaryTooLarge { size: usize, max: usize },
    #[error("invalid list structure")]
    InvalidList,
    #[error("invalid map structure")]
    InvalidMap,
    #[error("unsupported term type: {0}")]
    UnsupportedType(String),
    #[error("buffer too small: need {needed} bytes, have {available}")]
    BufferTooSmall { needed: usize, available: usize },
    #[error("invalid format: {0}")]
    InvalidFormat(String),
    #[error("trailing data: {0} bytes after term")]
    TrailingData(usize),
    #[error("invalid PID format: {0}")]
    InvalidPidFormat(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsingContext {
    pub byte_offset: usize,
    pub path: Vec<PathSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    TupleElement(usize),
    ListElement(usize),
    MapKey,
    MapValue(String),
    ImproperListTail,
    FunFreeVar(usize),
}

impl ParsingContext {
    pub fn new() -> Self {
        ParsingContext {
            byte_offset: 0,
            path: Vec::new(),
        }
    }

    pub fn with_offset(offset: usize) -> Self {
        ParsingContext {
            byte_offset: offset,
            path: Vec::new(),
        }
    }

    pub fn push(&mut self, segment: PathSegment) {
        self.path.push(segment);
    }

    pub fn pop(&mut self) {
        self.path.pop();
    }

    pub fn display_path(&self) -> String {
        if self.path.is_empty() {
            return "root".to_string();
        }

        let mut result = String::from("root");
        for segment in &self.path {
            match segment {
                PathSegment::TupleElement(i) => result.push_str(&format!("[{}]", i)),
                PathSegment::ListElement(i) => result.push_str(&format!("[{}]", i)),
                PathSegment::MapKey => result.push_str(".key"),
                PathSegment::MapValue(k) => result.push_str(&format!(".{}", k)),
                PathSegment::ImproperListTail => result.push_str(".tail"),
                PathSegment::FunFreeVar(i) => result.push_str(&format!(".free_var[{}]", i)),
            }
        }
        result
    }
}

impl Default for ParsingContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub struct ContextualDecodeError {
    pub error: DecodeError,
    pub context: ParsingContext,
}

impl fmt::Display for ContextualDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at byte offset {} in path {}",
            self.error,
            self.context.byte_offset,
            self.context.display_path()
        )
    }
}

impl ContextualDecodeError {
    pub fn new(error: DecodeError, context: ParsingContext) -> Self {
        ContextualDecodeError { error, context }
    }
}

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("atom too large: {size} bytes (max 65535)")]
    AtomTooLarge { size: usize },
    #[error("string too large: {size} bytes")]
    StringTooLarge { size: usize },
    #[error("list too large: {size} elements")]
    ListTooLarge { size: usize },
    #[error("map too large: {size} entries")]
    MapTooLarge { size: usize },
    #[error("binary too large: {size} bytes")]
    BinaryTooLarge { size: usize },
    #[error("tuple too large: {size} elements (max 4294967295)")]
    TupleTooLarge { size: usize },
    #[error("reference has too many IDs: {size} (max 65535)")]
    ReferenceTooLarge { size: usize },
    #[error("too many atoms for DIST_HEADER: {count} (max 255)")]
    TooManyAtoms { count: usize },
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("buffer overflow")]
    BufferOverflow,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TermConversionError {
    #[error("expected {expected}, got {actual}")]
    WrongType {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("value out of range for target type")]
    OutOfRange,
}

impl From<Utf8Error> for DecodeError {
    fn from(e: Utf8Error) -> Self {
        DecodeError::InvalidUtf8(e.to_string())
    }
}
