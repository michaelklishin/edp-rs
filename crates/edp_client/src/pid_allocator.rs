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

//! Process ID (PID) allocation for local processes.

use crate::errors::{Error, Result};
use crate::types::Creation;
use erltf::types::{Atom, ExternalPid};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

const MAX_PROCESSES_PER_NODE: u32 = 1_048_576;

#[derive(Debug)]
pub struct PidAllocator {
    node_name: Atom,
    creation: AtomicU32,
    next_id: AtomicU32,
    next_serial: AtomicU64,
    wrap_lock: Mutex<()>,
}

impl PidAllocator {
    pub fn new<C: Into<Creation>>(node_name: Atom, creation: C) -> Self {
        Self {
            node_name,
            creation: AtomicU32::new(creation.into().0),
            next_id: AtomicU32::new(1),
            next_serial: AtomicU64::new(0),
            wrap_lock: Mutex::new(()),
        }
    }

    pub fn allocate(&self) -> Result<ExternalPid> {
        let _guard = self.wrap_lock.lock().map_err(|e| {
            Error::InvalidStateMessage(format!("PID allocator lock poisoned: {}", e))
        })?;

        let id = self.next_id.load(Ordering::Relaxed);
        let serial_u64 = self.next_serial.load(Ordering::Relaxed);
        let serial = (serial_u64 % (u32::MAX as u64 + 1)) as u32;

        let next_id = id + 1;
        if id >= MAX_PROCESSES_PER_NODE {
            self.next_id.store(1, Ordering::Relaxed);
            let new_serial = self.next_serial.fetch_add(1, Ordering::Relaxed) + 1;
            let wrapped_serial = (new_serial % (u32::MAX as u64 + 1)) as u32;

            Ok(ExternalPid::new(
                self.node_name.clone(),
                id,
                wrapped_serial,
                self.creation.load(Ordering::Relaxed),
            ))
        } else {
            self.next_id.store(next_id, Ordering::Relaxed);

            Ok(ExternalPid::new(
                self.node_name.clone(),
                id,
                serial,
                self.creation.load(Ordering::Relaxed),
            ))
        }
    }

    pub fn node_name(&self) -> &Atom {
        &self.node_name
    }

    pub fn creation(&self) -> Creation {
        Creation(self.creation.load(Ordering::Relaxed))
    }

    /// Sets the creation counter. Read below before using this method.
    ///
    /// *Important*: this method should only be called when no allocations are in progress,
    /// typically during initialization or after receiving a new creation value
    /// from the remote node during the handshake.
    ///
    /// While the atomic store itself is safe for concurrent use, concurrent calls to `set_creation` during active allocation
    /// may result in PIDs being created with inconsistent creation values.
    pub fn set_creation<C: Into<Creation>>(&self, creation: C) {
        self.creation.store(creation.into().0, Ordering::Relaxed)
    }

    #[doc(hidden)]
    pub fn next_id_test_only(&self) -> &AtomicU32 {
        &self.next_id
    }

    #[doc(hidden)]
    pub fn next_serial_test_only(&self) -> &AtomicU64 {
        &self.next_serial
    }
}
