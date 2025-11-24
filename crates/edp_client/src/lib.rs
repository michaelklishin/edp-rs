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

//! Erlang Distribution Protocol (EDP) client implementation for Rust.
//!
//! This crate provides a client implementation of the Erlang Distribution Protocol,
//! targeting Erlang/OTP 26 and later versions. It enables Rust applications to
//! communicate with Erlang nodes using the native distribution protocol.
//!
//! # Features
//!
//! - Full protocol version 6 (OTP 23+) support
//! - Mandatory OTP 26+ capability flags
//! - EPMD (Erlang Port Mapper Daemon) client
//! - Async I/O using Tokio
//! - Type-safe message handling
//!
//! # Security
//!
//! The Erlang Distribution Protocol is not secure by itself. For production use:
//! - Use TLS distribution with mutual authentication
//! - Isolate distribution traffic on dedicated networks
//! - Do not expose EPMD or distribution ports publicly

pub mod connection;
pub mod control;
pub mod digest;
pub mod epmd_client;
pub mod errors;
pub mod flags;
pub mod fragmentation;
pub mod framing;
pub mod handshake;
pub mod pid_allocator;
pub mod state_machine;
pub mod term_helpers;
pub mod transport;
pub mod types;

pub use connection::{Connection, ConnectionConfig};
pub use errors::{Error, Result};
pub use flags::DistributionFlags;
pub use pid_allocator::PidAllocator;
pub use state_machine::ConnectionState;
pub use term_helpers::nil;
pub use tokio::net::tcp::OwnedReadHalf;
pub use types::{Creation, SequenceId};
