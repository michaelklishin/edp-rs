// Copyright (C) 2025-2026 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::state_machine::ConnectionState;
use erltf::errors::{ContextualDecodeError, DecodeError, EncodeError, TermConversionError};
use std::io;
use std::time::Duration;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Connection timeout after {0:?}")]
    Timeout(Duration),

    #[error("Connection closed by peer")]
    ConnectionClosed,

    #[error("Connection refused by peer: {reason}")]
    ConnectionRefused { reason: String },

    #[error("EPMD lookup failed for node '{node}': {reason}")]
    EpmdLookup { node: String, reason: String },

    #[error("EPMD registration failed: {reason}")]
    EpmdRegistration { reason: String },

    #[error("EPMD protocol error: {0}")]
    EpmdProtocol(String),

    #[error("Handshake failed: {reason}")]
    HandshakeFailed { reason: String },

    #[error("Authentication failed: challenge validation mismatch")]
    AuthenticationFailed,

    #[error("Incompatible protocol version: got {got}, expected {expected}")]
    IncompatibleVersion { got: u16, expected: u16 },

    #[error("Missing mandatory capability flags: {missing:?}")]
    MissingMandatoryFlags { missing: Vec<String> },

    #[error("Invalid handshake message: {0}")]
    InvalidHandshakeMessage(String),

    #[error("Invalid control message: {0}")]
    InvalidControlMessage(String),

    #[error("Message too large: {size} bytes (max {max} bytes)")]
    MessageTooLarge { size: usize, max: usize },

    #[error("Node name too long: {size} bytes (max {max} bytes)")]
    NodeNameTooLong { size: usize, max: usize },

    #[error("Invalid node name: {0}")]
    InvalidNodeName(String),

    #[error("Invalid atom: {0}")]
    InvalidAtom(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),

    #[error("Decode error: {0}")]
    Decode(#[from] DecodeError),

    #[error("Contextual decode error: {0}")]
    ContextualDecode(#[from] ContextualDecodeError),

    #[error("Term conversion error: {0}")]
    TermConversion(#[from] TermConversionError),

    #[error("Unexpected EOF while reading {context}")]
    UnexpectedEof { context: String },

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition {
        from: ConnectionState,
        to: ConnectionState,
    },

    #[error("Operation not supported in current state: {state}")]
    InvalidState { state: ConnectionState },

    #[error("{0}")]
    InvalidStateMessage(String),
}

impl Error {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::Io(_) | Error::Timeout(_) | Error::UnexpectedEof { .. }
        )
    }

    pub fn is_connection_closed(&self) -> bool {
        match self {
            Error::ConnectionClosed | Error::UnexpectedEof { .. } => true,
            Error::Io(e) => {
                matches!(
                    e.kind(),
                    io::ErrorKind::UnexpectedEof
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::BrokenPipe
                )
            }
            _ => false,
        }
    }

    pub fn is_timeout(&self) -> bool {
        match self {
            Error::Timeout(_) => true,
            Error::Io(e) => e.kind() == io::ErrorKind::TimedOut,
            _ => false,
        }
    }
}
