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

use edp_client::Error as ClientError;
use erltf::EncodeError;
use erltf::types::{Atom, ExternalPid};
use std::time::Duration;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("Encode error: {0}")]
    Encode(#[from] EncodeError),

    #[error("Process not found: {0:?}")]
    ProcessNotFound(ExternalPid),

    #[error("Process name not registered: {0}")]
    NameNotRegistered(Atom),

    #[error("Process name already registered: {0}")]
    NameAlreadyRegistered(Atom),

    #[error("Remote node not connected: {0}")]
    NodeNotConnected(String),

    #[error("Call timeout after {0:?}")]
    CallTimeout(Duration),

    #[error("Mailbox closed")]
    MailboxClosed,

    #[error("Spawn failed: {0}")]
    SpawnFailed(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Node already started")]
    NodeAlreadyStarted,

    #[error("Node not started")]
    NodeNotStarted,

    #[error("EPMD registration failed: {0}")]
    EpmdRegistration(String),

    #[error("RPC timeout")]
    RpcTimeout,

    #[error("RPC cancelled")]
    RpcCancelled,
}
