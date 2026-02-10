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

//! High-level Erlang node abstraction with process management.
//!
//! This crate provides a high-level API for creating Erlang distribution protocol
//! nodes that can spawn processes, send messages, and communicate with remote nodes.
//!
//! # Features
//!
//! - Process spawning and management
//! - Process registration by name
//! - Message routing to local and remote processes
//! - GenServer behavior pattern
//! - Process linking and monitoring
//!
//! # Example
//!
//! ```no_run
//! use edp_node::{Node, Process, Message, Result};
//! use erltf::OwnedTerm;
//!
//! struct MyProcess;
//!
//! impl Process for MyProcess {
//!     async fn handle_message(&mut self, msg: Message) -> Result<()> {
//!         println!("Received: {:?}", msg);
//!         Ok(())
//!     }
//! }
//! ```

pub mod erlang_mod_fns;
pub mod errors;
pub mod gen_event;
pub mod gen_server;
pub mod mailbox;
pub mod node;
pub mod process;
pub mod registry;

pub use errors::{Error, Result};
pub use gen_event::{
    CallResult as GenEventCallResult, EventResult, GenEventHandler, GenEventManager,
};
pub use gen_server::{CallResult, GenServer, GenServerProcess};
pub use mailbox::{Mailbox, Message};
pub use node::{
    DEFAULT_CONNECT_RETRY_ATTEMPTS, DEFAULT_CONNECT_RETRY_DELAY, DEFAULT_RPC_TIMEOUT, Node,
};
pub use process::{Process, ProcessHandle};
pub use registry::ProcessRegistry;

pub use erltf::{
    Atom, ExternalPid, Mfa, OwnedTerm, erl_atom, erl_atoms, erl_int, erl_list, erl_map, erl_tuple,
    errors::TermConversionError,
};
pub use erltf_serde::{OwnedTermExt, from_term, to_term};
