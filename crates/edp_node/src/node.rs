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
use crate::mailbox::{Mailbox, Message};
use crate::process::{Process, spawn_process};
use crate::registry::ProcessRegistry;
use dashmap::DashMap;
use edp_client::control::ControlMessage;
use edp_client::epmd_client::{EpmdClient, NodeType};
use edp_client::{Connection, ConnectionConfig, PidAllocator};
use erltf::OwnedTerm;
use erltf::types::{Atom, ExternalPid, ExternalReference};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;
use tokio::sync::{Mutex, oneshot};

pub const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Node {
    name: Atom,
    cookie: String,
    creation: Arc<AtomicU32>,
    pid_allocator: Arc<PidAllocator>,
    reference_counter: Arc<AtomicU32>,
    registry: Arc<ProcessRegistry>,
    connections: Arc<DashMap<String, Arc<Mutex<Connection>>>>,
    pending_rpcs: Arc<DashMap<String, oneshot::Sender<OwnedTerm>>>,
    started: Arc<AtomicBool>,
    listen_port: Option<u16>,
    hidden: bool,
}

impl Node {
    pub fn new(name: impl Into<String>, cookie: impl Into<String>) -> Self {
        Self::with_hidden(name, cookie, false)
    }

    pub fn new_hidden(name: impl Into<String>, cookie: impl Into<String>) -> Self {
        Self::with_hidden(name, cookie, true)
    }

    pub async fn connect_to(
        name: impl Into<String>,
        cookie: impl Into<String>,
        remote_node: impl Into<String>,
    ) -> Result<Self> {
        Self::connect_to_with_hidden(name, cookie, remote_node, false).await
    }

    pub async fn connect_to_hidden(
        name: impl Into<String>,
        cookie: impl Into<String>,
        remote_node: impl Into<String>,
    ) -> Result<Self> {
        Self::connect_to_with_hidden(name, cookie, remote_node, true).await
    }

    async fn connect_to_with_hidden(
        name: impl Into<String>,
        cookie: impl Into<String>,
        remote_node: impl Into<String>,
        hidden: bool,
    ) -> Result<Self> {
        let mut node = Self::with_hidden(name, cookie, hidden);
        node.start(0).await?;
        node.connect(remote_node).await?;
        Ok(node)
    }

    fn with_hidden(name: impl Into<String>, cookie: impl Into<String>, hidden: bool) -> Self {
        let name_atom = Atom::new(name.into());
        let creation = 1;
        let pid_allocator = Arc::new(PidAllocator::new(name_atom.clone(), creation));
        let creation = Arc::new(AtomicU32::new(creation));

        Self {
            name: name_atom,
            cookie: cookie.into(),
            creation,
            pid_allocator,
            reference_counter: Arc::new(AtomicU32::new(0)),
            registry: Arc::new(ProcessRegistry::new()),
            connections: Arc::new(DashMap::new()),
            pending_rpcs: Arc::new(DashMap::new()),
            started: Arc::new(AtomicBool::new(false)),
            listen_port: None,
            hidden,
        }
    }

    pub fn registry(&self) -> Arc<ProcessRegistry> {
        self.registry.clone()
    }

    pub async fn start(&mut self, port: u16) -> Result<()> {
        if self.started.swap(true, Ordering::SeqCst) {
            return Err(Error::NodeAlreadyStarted);
        }

        let (node_name, _host) =
            self.name.as_str().split_once('@').ok_or_else(|| {
                Error::EpmdRegistration(format!("Invalid node name: {}", self.name))
            })?;

        let epmd = EpmdClient::new("localhost");
        let creation = epmd
            .register_node(port, node_name, NodeType::Normal, 6, 6, &[])
            .await
            .map_err(|e| Error::EpmdRegistration(e.to_string()))?;

        self.creation.store(creation, Ordering::SeqCst);
        self.pid_allocator.set_creation(creation);
        self.listen_port = Some(port);

        tracing::info!(
            "Node {} started on port {} with creation {}",
            self.name,
            port,
            creation
        );
        Ok(())
    }

    pub async fn connect(&self, remote_node: impl Into<String>) -> Result<()> {
        let remote_node = remote_node.into();

        if self.connections.contains_key(&remote_node) {
            return Ok(());
        }

        let config = if self.hidden {
            ConnectionConfig::new_hidden(self.name.as_str(), &remote_node, &self.cookie)
        } else {
            ConnectionConfig::new(self.name.as_str(), &remote_node, &self.cookie)
        };

        let mut conn = Connection::new(config);
        conn.connect().await?;

        let read_half = conn.take_read_half().ok_or_else(|| {
            edp_client::Error::InvalidStateMessage(
                "Failed to take read half from connection".to_string(),
            )
        })?;

        let timeout = conn.timeout();

        self.connections
            .insert(remote_node.clone(), Arc::new(Mutex::new(conn)));

        self.spawn_receiver_task(remote_node.clone(), read_half, timeout);

        tracing::info!("Connected to {}", remote_node);
        Ok(())
    }

    fn spawn_receiver_task(
        &self,
        remote_node: String,
        mut read_half: edp_client::OwnedReadHalf,
        timeout: std::time::Duration,
    ) {
        let registry = self.registry.clone();
        let pending_rpcs = self.pending_rpcs.clone();
        let connections = self.connections.clone();
        let remote_node_clone = remote_node.clone();

        tokio::spawn(async move {
            loop {
                let result =
                    edp_client::Connection::receive_message_from_read_half(&mut read_half, timeout)
                        .await;

                match result {
                    Ok((control_msg, payload)) => {
                        let payload_len = payload.as_ref().map(|p| p.len()).unwrap_or(0);
                        tracing::debug!(
                            "Received control message from {}, payload size: {} bytes",
                            remote_node,
                            payload_len
                        );
                        tracing::debug!(
                            "Control message details: {:?}, payload: {:?}",
                            control_msg,
                            payload
                        );
                        if let Err(e) =
                            Self::route_message(&registry, &pending_rpcs, control_msg, payload)
                                .await
                        {
                            tracing::error!("Failed to route message: {}", e);
                        }
                    }
                    Err(e) => {
                        if e.to_string().contains("Decode error") {
                            tracing::warn!(
                                "Failed to decode message from {} (likely unsupported message type): {}",
                                remote_node,
                                e
                            );
                            continue;
                        }
                        tracing::error!("Error receiving message from {}: {}", remote_node, e);
                        break;
                    }
                }
            }

            connections.remove(&remote_node_clone);
            tracing::info!(
                "Receiver task for {} terminated, connection removed",
                remote_node
            );
        });
    }

    async fn route_message(
        registry: &ProcessRegistry,
        pending_rpcs: &DashMap<String, oneshot::Sender<OwnedTerm>>,
        control_msg: ControlMessage,
        payload: Option<OwnedTerm>,
    ) -> Result<()> {
        match control_msg {
            ControlMessage::Send { to_pid, .. } => {
                if let Some(body) = payload
                    && let OwnedTerm::Pid(pid) = to_pid
                {
                    if let Some(handle) = registry.get(&pid).await {
                        handle.send(Message::Regular { from: None, body }).await?;
                    } else {
                        let pid_str = format!("{}.{}.{}", pid.id, pid.serial, pid.creation);
                        if let Some((_key, sender)) = pending_rpcs.remove(&pid_str) {
                            let _ = sender.send(body);
                        }
                    }
                }
            }
            ControlMessage::RegSend { to_name, .. } => {
                if let Some(body) = payload
                    && let OwnedTerm::Atom(name) = to_name
                    && let Some(pid) = registry.whereis(&name).await
                    && let Some(handle) = registry.get(&pid).await
                {
                    handle.send(Message::Regular { from: None, body }).await?;
                }
            }
            ControlMessage::Exit {
                from_pid,
                to_pid,
                reason,
            } => {
                if let OwnedTerm::Pid(from) = from_pid
                    && let OwnedTerm::Pid(to) = to_pid
                    && let Some(handle) = registry.get(&to).await
                {
                    handle.send(Message::Exit { from, reason }).await?;
                }
            }
            ControlMessage::MonitorPExit {
                from_proc,
                to_pid,
                reference,
                reason,
            } => {
                if let OwnedTerm::Pid(from) = from_proc
                    && let OwnedTerm::Pid(to) = to_pid
                    && let OwnedTerm::Reference(ref_val) = reference
                    && let Some(handle) = registry.get(&to).await
                {
                    handle
                        .send(Message::MonitorExit {
                            monitored: from,
                            reference: ref_val,
                            reason,
                        })
                        .await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn spawn<P: Process>(&self, process: P) -> Result<ExternalPid> {
        if !self.started.load(Ordering::SeqCst) {
            return Err(Error::NodeNotStarted);
        }

        let mailbox = Mailbox::new();
        let pid = self
            .pid_allocator
            .allocate()
            .expect("PID allocator lock poisoned");

        let handle = spawn_process(process, mailbox, self.registry.clone(), pid.clone()).await;

        self.registry.insert(pid.clone(), handle).await;

        tracing::debug!("Spawned process: {:?}", pid);
        Ok(pid)
    }

    pub async fn register(&self, name: Atom, pid: ExternalPid) -> Result<()> {
        self.registry.register(name, pid).await
    }

    pub async fn unregister(&self, name: &Atom) -> Result<()> {
        self.registry.unregister(name).await
    }

    pub async fn whereis(&self, name: &Atom) -> Option<ExternalPid> {
        self.registry.whereis(name).await
    }

    pub async fn registered(&self) -> Vec<Atom> {
        self.registry.registered().await
    }

    pub async fn send(&self, to: &ExternalPid, message: OwnedTerm) -> Result<()> {
        if to.node == self.name {
            self.send_local(to, message).await
        } else {
            self.send_remote(to, message).await
        }
    }

    pub async fn send_to_name(&self, to: &Atom, message: OwnedTerm) -> Result<()> {
        let pid = self
            .whereis(to)
            .await
            .ok_or_else(|| Error::NameNotRegistered(to.clone()))?;
        self.send(&pid, message).await
    }

    async fn send_local(&self, to: &ExternalPid, message: OwnedTerm) -> Result<()> {
        if let Some(handle) = self.registry.get(to).await {
            handle
                .send(Message::Regular {
                    from: None,
                    body: message,
                })
                .await?;
            Ok(())
        } else {
            Err(Error::ProcessNotFound(to.clone()))
        }
    }

    async fn send_remote(&self, to: &ExternalPid, message: OwnedTerm) -> Result<()> {
        let node_name = to.node.as_str();

        if let Some(conn) = self.connections.get(node_name) {
            let from = self
                .pid_allocator
                .allocate()
                .expect("PID allocator lock poisoned");
            let mut conn_guard = conn.lock().await;
            conn_guard.send_message(from, to.clone(), message).await?;
            Ok(())
        } else {
            Err(Error::NodeNotConnected(node_name.to_string()))
        }
    }

    pub async fn link(&self, from: &ExternalPid, to: &ExternalPid) -> Result<()> {
        if let Some(from_handle) = self.registry.get(from).await {
            from_handle.add_link(to.clone()).await;
        }

        if to.node == self.name {
            if let Some(to_handle) = self.registry.get(to).await {
                to_handle.add_link(from.clone()).await;
            }
            Ok(())
        } else {
            let node_name = to.node.as_str();

            if let Some(conn) = self.connections.get(node_name) {
                let mut conn_guard = conn.lock().await;
                conn_guard.link(from, to).await?;
                Ok(())
            } else {
                Err(Error::NodeNotConnected(node_name.to_string()))
            }
        }
    }

    pub async fn unlink(&self, from: &ExternalPid, to: &ExternalPid) -> Result<()> {
        if let Some(from_handle) = self.registry.get(from).await {
            from_handle.remove_link(to).await;
        }

        if to.node == self.name {
            if let Some(to_handle) = self.registry.get(to).await {
                to_handle.remove_link(from).await;
            }
            Ok(())
        } else {
            let node_name = to.node.as_str();

            if let Some(conn) = self.connections.get(node_name) {
                let unlink_id = self.reference_counter.fetch_add(1, Ordering::SeqCst) as u64;
                let mut conn_guard = conn.lock().await;
                conn_guard.unlink(from, to, unlink_id).await?;
                Ok(())
            } else {
                Err(Error::NodeNotConnected(node_name.to_string()))
            }
        }
    }

    pub fn make_reference(&self) -> ExternalReference {
        let id0 = self.reference_counter.fetch_add(1, Ordering::SeqCst);
        let id1 = self.reference_counter.fetch_add(1, Ordering::SeqCst);
        let id2 = self.reference_counter.fetch_add(1, Ordering::SeqCst);
        ExternalReference::new(
            self.name.clone(),
            self.creation.load(Ordering::SeqCst),
            vec![id0, id1, id2],
        )
    }

    pub async fn monitor(&self, from: &ExternalPid, to: &ExternalPid) -> Result<ExternalReference> {
        let reference = self.make_reference();

        if to.node == self.name {
            if let Some(to_handle) = self.registry.get(to).await {
                to_handle.add_monitor(from.clone(), reference.clone()).await;
            }
            Ok(reference)
        } else {
            let node_name = to.node.as_str();

            if let Some(conn) = self.connections.get(node_name) {
                let mut conn_guard = conn.lock().await;
                conn_guard.monitor(from, to, &reference).await?;
                Ok(reference)
            } else {
                Err(Error::NodeNotConnected(node_name.to_string()))
            }
        }
    }

    pub async fn demonitor(
        &self,
        from: &ExternalPid,
        to: &ExternalPid,
        reference: &ExternalReference,
    ) -> Result<()> {
        if to.node == self.name {
            if let Some(to_handle) = self.registry.get(to).await {
                to_handle.remove_monitor(reference).await;
            }
            Ok(())
        } else {
            let node_name = to.node.as_str();

            if let Some(conn) = self.connections.get(node_name) {
                let mut conn_guard = conn.lock().await;
                conn_guard.demonitor(from, to, reference).await?;
                Ok(())
            } else {
                Err(Error::NodeNotConnected(node_name.to_string()))
            }
        }
    }

    pub fn name(&self) -> &Atom {
        &self.name
    }

    pub fn creation(&self) -> u32 {
        self.creation.load(Ordering::SeqCst)
    }

    pub async fn process_count(&self) -> usize {
        self.registry.count().await
    }

    pub fn connections(&self) -> Arc<DashMap<String, Arc<Mutex<Connection>>>> {
        self.connections.clone()
    }

    pub fn cookie(&self) -> &str {
        &self.cookie
    }

    pub async fn rpc_call(
        &self,
        remote_node: &str,
        module: &str,
        function: &str,
        args: Vec<OwnedTerm>,
    ) -> Result<OwnedTerm> {
        self.rpc_call_with_timeout(remote_node, module, function, args, DEFAULT_RPC_TIMEOUT)
            .await
    }

    pub async fn rpc_call_with_timeout(
        &self,
        remote_node: &str,
        module: &str,
        function: &str,
        args: Vec<OwnedTerm>,
        timeout: Duration,
    ) -> Result<OwnedTerm> {
        let response = self
            .rpc_call_raw_with_timeout(remote_node, module, function, args, timeout)
            .await?;
        response.into_rex_response().map_err(Error::from)
    }

    pub async fn rpc_call_raw(
        &self,
        remote_node: &str,
        module: &str,
        function: &str,
        args: Vec<OwnedTerm>,
    ) -> Result<OwnedTerm> {
        self.rpc_call_raw_with_timeout(remote_node, module, function, args, DEFAULT_RPC_TIMEOUT)
            .await
    }

    pub async fn rpc_call_raw_with_timeout(
        &self,
        remote_node: &str,
        module: &str,
        function: &str,
        args: Vec<OwnedTerm>,
        timeout: Duration,
    ) -> Result<OwnedTerm> {
        let reply_to_pid = self
            .pid_allocator
            .allocate()
            .expect("PID allocator lock poisoned");

        let call_request = OwnedTerm::Tuple(vec![
            OwnedTerm::Pid(reply_to_pid.clone()),
            OwnedTerm::Tuple(vec![
                OwnedTerm::Atom(Atom::new("call")),
                OwnedTerm::Atom(Atom::new(module)),
                OwnedTerm::Atom(Atom::new(function)),
                OwnedTerm::List(args),
                OwnedTerm::Atom(Atom::new("user")),
            ]),
        ]);

        let (tx, rx) = oneshot::channel();
        let pid_str = format!(
            "{}.{}.{}",
            reply_to_pid.id, reply_to_pid.serial, reply_to_pid.creation
        );
        self.pending_rpcs.insert(pid_str.clone(), tx);

        tracing::debug!("RPC call_request: {:?}", call_request);
        tracing::debug!("RPC reply_to_pid: {:?}", reply_to_pid);

        tracing::trace!("Looking up connection for node: {}", remote_node);
        if let Some(conn) = self.connections.get(remote_node) {
            tracing::trace!("Found connection, sending to rex");
            let mut conn_guard = conn.lock().await;
            conn_guard
                .send_to_name(reply_to_pid, Atom::new("rex"), call_request)
                .await?;
            tracing::trace!("Message sent to rex");
        } else {
            tracing::error!("No connection found for node: {}", remote_node);
            self.pending_rpcs.remove(&pid_str);
            return Err(Error::NodeNotConnected(remote_node.to_string()));
        }

        let response = tokio::time::timeout(timeout, rx).await;

        if response.is_err() {
            self.pending_rpcs.remove(&pid_str);
        }

        let response = response
            .map_err(|_| Error::RpcTimeout(timeout))?
            .map_err(|_| Error::RpcCancelled)?;

        Ok(response)
    }
}
