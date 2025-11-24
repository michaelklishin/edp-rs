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

use crate::errors::{Error, Result};
use crate::process::ProcessHandle;
use erltf::types::{Atom, ExternalPid};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ProcessRegistry {
    by_pid: Arc<RwLock<HashMap<ExternalPid, ProcessHandle>>>,
    by_name: Arc<RwLock<HashMap<Atom, ExternalPid>>>,
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self {
            by_pid: Arc::new(RwLock::new(HashMap::new())),
            by_name: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, pid: ExternalPid, handle: ProcessHandle) {
        self.by_pid.write().await.insert(pid, handle);
    }

    pub async fn remove(&self, pid: &ExternalPid) -> Option<ProcessHandle> {
        self.by_pid.write().await.remove(pid)
    }

    pub async fn get(&self, pid: &ExternalPid) -> Option<ProcessHandle> {
        self.by_pid.read().await.get(pid).cloned()
    }

    pub async fn register(&self, name: Atom, pid: ExternalPid) -> Result<()> {
        let mut names = self.by_name.write().await;
        match names.entry(name.clone()) {
            Entry::Occupied(_) => Err(Error::NameAlreadyRegistered(name)),
            Entry::Vacant(e) => {
                e.insert(pid);
                Ok(())
            }
        }
    }

    pub async fn unregister(&self, name: &Atom) -> Result<()> {
        self.by_name
            .write()
            .await
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| Error::NameNotRegistered(name.clone()))
    }

    pub async fn whereis(&self, name: &Atom) -> Option<ExternalPid> {
        self.by_name.read().await.get(name).cloned()
    }

    pub async fn registered(&self) -> Vec<Atom> {
        self.by_name.read().await.keys().cloned().collect()
    }

    pub async fn count(&self) -> usize {
        self.by_pid.read().await.len()
    }
}

impl Default for ProcessRegistry {
    fn default() -> Self {
        Self::new()
    }
}
