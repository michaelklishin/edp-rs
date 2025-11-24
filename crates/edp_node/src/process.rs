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

use crate::errors::Result;
use crate::mailbox::{Mailbox, Message};
use crate::registry::ProcessRegistry;
use erltf::OwnedTerm;
use erltf::types::{Atom, ExternalPid, ExternalReference};
use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

pub trait Process: Send + 'static {
    fn handle_message(&mut self, msg: Message) -> impl Future<Output = Result<()>> + Send + '_;

    fn terminate(&mut self) -> impl Future<Output = ()> + Send + '_ {
        async {}
    }
}

#[derive(Clone)]
pub struct ProcessHandle {
    pub pid: ExternalPid,
    pub mailbox_sender: mpsc::Sender<Message>,
    links: Arc<RwLock<HashSet<ExternalPid>>>,
    monitors: Arc<RwLock<HashSet<(ExternalPid, ExternalReference)>>>,
}

impl ProcessHandle {
    pub fn new(pid: ExternalPid, mailbox_sender: mpsc::Sender<Message>) -> Self {
        Self {
            pid,
            mailbox_sender,
            links: Arc::new(RwLock::new(HashSet::new())),
            monitors: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn send(&self, msg: Message) -> Result<()> {
        self.mailbox_sender
            .send(msg)
            .await
            .map_err(|_| crate::errors::Error::MailboxClosed)
    }

    pub async fn add_link(&self, other_pid: ExternalPid) {
        self.links.write().await.insert(other_pid);
    }

    pub async fn remove_link(&self, other_pid: &ExternalPid) {
        self.links.write().await.remove(other_pid);
    }

    pub async fn add_monitor(&self, monitoring_pid: ExternalPid, reference: ExternalReference) {
        self.monitors
            .write()
            .await
            .insert((monitoring_pid, reference));
    }

    pub async fn remove_monitor(&self, reference: &ExternalReference) {
        self.monitors.write().await.retain(|(_, r)| r != reference);
    }

    pub async fn get_links(&self) -> Vec<ExternalPid> {
        self.links.read().await.iter().cloned().collect()
    }

    pub async fn get_monitors(&self) -> Vec<(ExternalPid, ExternalReference)> {
        self.monitors.read().await.iter().cloned().collect()
    }
}

pub async fn spawn_process<P: Process>(
    mut process: P,
    mut mailbox: Mailbox,
    registry: Arc<ProcessRegistry>,
    pid: ExternalPid,
) -> ProcessHandle {
    let sender = mailbox.sender();
    let handle = ProcessHandle::new(pid.clone(), sender);
    let handle_clone = handle.clone();

    tokio::spawn(async move {
        let exit_reason = loop {
            match mailbox.recv().await {
                Ok(msg) => {
                    if let Err(e) = process.handle_message(msg).await {
                        tracing::error!("Process {} error: {}", pid, e);
                        break OwnedTerm::Atom(Atom::new("error"));
                    }
                }
                Err(_) => {
                    break OwnedTerm::Atom(Atom::new("normal"));
                }
            }
        };

        process.terminate().await;

        if let Err(e) = propagate_exit_signals(&handle_clone, &registry, exit_reason).await {
            tracing::error!("Failed to propagate exit signals for {}: {}", pid, e);
        }

        registry.remove(&pid).await;
    });

    handle
}

async fn propagate_exit_signals(
    handle: &ProcessHandle,
    registry: &ProcessRegistry,
    reason: OwnedTerm,
) -> Result<()> {
    let links = handle.get_links().await;
    for linked_pid in links {
        if let Some(linked_handle) = registry.get(&linked_pid).await {
            let _ = linked_handle
                .send(Message::Exit {
                    from: handle.pid.clone(),
                    reason: reason.clone(),
                })
                .await;
        }
    }

    let monitors = handle.get_monitors().await;
    for (monitoring_pid, reference) in monitors {
        if let Some(monitoring_handle) = registry.get(&monitoring_pid).await {
            let _ = monitoring_handle
                .send(Message::MonitorExit {
                    monitored: handle.pid.clone(),
                    reference,
                    reason: reason.clone(),
                })
                .await;
        }
    }

    Ok(())
}
