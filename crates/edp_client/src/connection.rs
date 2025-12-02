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

//! Distribution protocol connection orchestration.

use crate::control::ControlMessage;
use crate::epmd_client::EpmdClient;
use crate::errors::{Error, Result};
use crate::flags::DistributionFlags;
use crate::fragmentation::FragmentAssembler;
use crate::framing::FrameMode;
use crate::state_machine::{ConnectionState, HandshakeStateMachine};
use crate::transport::FramedTransport;
use crate::types::Creation;
use bytes::{BufMut, BytesMut};
use erltf::decoder::AtomCache;
use erltf::types::{Atom, ExternalPid, ExternalReference};
use erltf::{OwnedTerm, decoder};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tracing::{debug, trace};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

const VERSION_TAG: u8 = 131;
const DIST_HEADER: u8 = 68;
const DIST_FRAG_HEADER: u8 = 69;
const DIST_FRAG_CONT: u8 = 70;
const PASS_THROUGH: u8 = 112;

pub struct ConnectionConfig {
    pub local_node_name: String,
    pub remote_node_name: String,
    pub cookie: String,
    pub epmd_host: String,
    pub flags: DistributionFlags,
    pub creation: Creation,
    pub timeout: Duration,
}

impl ConnectionConfig {
    pub fn new(
        local_node_name: impl Into<String>,
        remote_node_name: impl Into<String>,
        cookie: impl Into<String>,
    ) -> Self {
        Self {
            local_node_name: local_node_name.into(),
            remote_node_name: remote_node_name.into(),
            cookie: cookie.into(),
            epmd_host: "localhost".to_string(),
            flags: DistributionFlags::default(),
            creation: Creation::default(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn new_hidden(
        local_node_name: impl Into<String>,
        remote_node_name: impl Into<String>,
        cookie: impl Into<String>,
    ) -> Self {
        Self {
            local_node_name: local_node_name.into(),
            remote_node_name: remote_node_name.into(),
            cookie: cookie.into(),
            epmd_host: "localhost".to_string(),
            flags: DistributionFlags::default_hidden(),
            creation: Creation::default(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn with_epmd_host(mut self, host: impl Into<String>) -> Self {
        self.epmd_host = host.into();
        self
    }

    pub fn with_flags(mut self, flags: DistributionFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_creation<C: Into<Creation>>(mut self, creation: C) -> Self {
        self.creation = creation.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

pub struct Connection {
    config: ConnectionConfig,
    handshake: HandshakeStateMachine,
    transport: FramedTransport,
    atom_cache: AtomCache,
    fragment_assembler: FragmentAssembler,
}

impl Connection {
    pub fn new(config: ConnectionConfig) -> Self {
        let handshake = HandshakeStateMachine::new(
            config.local_node_name.clone(),
            config.remote_node_name.clone(),
            config.cookie.clone(),
            config.flags,
            config.creation,
        );
        let transport = FramedTransport::new(config.timeout);

        Self {
            config,
            handshake,
            transport,
            atom_cache: AtomCache::new(),
            fragment_assembler: FragmentAssembler::new(),
        }
    }

    #[must_use]
    pub fn state(&self) -> ConnectionState {
        self.handshake.state()
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.handshake.state() == ConnectionState::Connected
    }

    #[must_use]
    pub fn negotiated_flags(&self) -> Option<DistributionFlags> {
        self.handshake.negotiated_flags()
    }

    fn validate_node_name(name: &str) -> Result<(&str, &str)> {
        let (node_name, host) = name
            .split_once('@')
            .ok_or_else(|| Error::InvalidNodeName(name.to_string()))?;

        if node_name.is_empty() || host.is_empty() {
            return Err(Error::InvalidNodeName(name.to_string()));
        }

        if node_name.len() > 255 {
            return Err(Error::NodeNameTooLong {
                size: node_name.len(),
                max: 255,
            });
        }

        Ok((node_name, host))
    }

    async fn lookup_remote_node(&self) -> Result<u16> {
        let epmd = EpmdClient::new(&self.config.epmd_host).with_timeout(self.config.timeout);

        let (node_name, _host) = Self::validate_node_name(&self.config.remote_node_name)?;

        let node_info = epmd.lookup_node(node_name).await?;
        debug!(
            "EPMD node info: port={}, highest_version={}, lowest_version={}",
            node_info.port, node_info.highest_version, node_info.lowest_version
        );
        Ok(node_info.port)
    }

    async fn read_message(&mut self) -> Result<Vec<u8>> {
        self.transport.read().await
    }

    async fn write_message(&mut self, data: &[u8]) -> Result<()> {
        self.transport.write(data).await
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.handshake.begin_connect()?;
        debug!("Connection state: {:?}", self.state());

        let (_node_name, remote_host) = self
            .config
            .remote_node_name
            .split_once('@')
            .ok_or_else(|| Error::InvalidNodeName(self.config.remote_node_name.clone()))?;

        debug!(
            "Looking up node via EPMD on host: {}",
            self.config.epmd_host
        );
        let port = self.lookup_remote_node().await?;
        debug!("EPMD returned port: {}", port);

        let addr = format!("{}:{}", remote_host, port);
        debug!("Connecting to: {}", addr);

        let stream = tokio::time::timeout(self.config.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| Error::Timeout(self.config.timeout))?
            .map_err(Error::Io)?;

        debug!("TCP connection established");
        self.transport.connect(stream);

        debug!("Starting handshake sequence");
        self.send_name().await?;
        self.receive_status().await?;
        self.send_complement().await?;
        self.receive_challenge().await?;
        self.send_challenge_reply().await?;
        self.receive_challenge_ack().await?;

        self.transport.set_frame_mode(FrameMode::Distribution);
        debug!("Handshake complete, connection established");

        Ok(())
    }

    async fn send_name(&mut self) -> Result<()> {
        debug!("Sending name: {}", self.config.local_node_name);
        let data = self.handshake.prepare_send_name()?;
        trace!("SendName (old format) encoded: {} bytes", data.len());
        trace!("SendName data (hex): {:02x?}", data);
        self.transport.write_raw(&data).await?;
        debug!("Name sent, awaiting status");
        Ok(())
    }

    async fn send_complement(&mut self) -> Result<()> {
        debug!("Sending complement message");
        let data = self.handshake.prepare_complement()?;
        trace!("Complement data (hex): {:02x?}", data);
        self.transport.write_raw(&data).await?;
        debug!("Complement sent");
        Ok(())
    }

    async fn receive_status(&mut self) -> Result<()> {
        debug!("Waiting for status message");
        let data = self.read_message().await?;
        trace!("Received status message: {} bytes", data.len());
        self.handshake.handle_status(&data)?;
        Ok(())
    }

    async fn receive_challenge(&mut self) -> Result<()> {
        let data = self.read_message().await?;
        self.handshake.handle_challenge(&data)?;
        debug!("Negotiated flags: {:?}", self.handshake.negotiated_flags());
        Ok(())
    }

    async fn send_challenge_reply(&mut self) -> Result<()> {
        debug!("Sending challenge reply");
        let data = self.handshake.prepare_challenge_reply()?;
        trace!("Challenge reply data (hex): {:02x?}", data);
        self.transport.write_raw(&data).await?;
        Ok(())
    }

    async fn receive_challenge_ack(&mut self) -> Result<()> {
        let data = self.read_message().await?;
        self.handshake.handle_challenge_ack(&data)?;
        Ok(())
    }

    pub async fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        if self.state() != ConnectionState::Connected {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        if data.len() > MAX_MESSAGE_SIZE {
            return Err(Error::MessageTooLarge {
                size: data.len(),
                max: MAX_MESSAGE_SIZE,
            });
        }

        self.write_message(data).await
    }

    pub async fn receive_raw(&mut self) -> Result<Vec<u8>> {
        if self.state() != ConnectionState::Connected {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        self.read_message().await
    }

    pub async fn close(&mut self) -> Result<()> {
        self.transport.close();
        self.handshake.disconnect();
        Ok(())
    }

    pub async fn send_message(
        &mut self,
        _from_pid: ExternalPid,
        to_pid: ExternalPid,
        message: OwnedTerm,
    ) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        let control = ControlMessage::Send {
            cookie: OwnedTerm::Atom(Atom::new("")),
            to_pid: OwnedTerm::Pid(to_pid),
        };

        self.send_control_message(control, Some(message)).await
    }

    pub async fn send_to_name(
        &mut self,
        from_pid: ExternalPid,
        to_name: Atom,
        message: OwnedTerm,
    ) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        let control = ControlMessage::RegSend {
            from_pid: OwnedTerm::Pid(from_pid),
            cookie: OwnedTerm::Atom(Atom::new("")),
            to_name: OwnedTerm::Atom(to_name),
        };

        self.send_control_message(control, Some(message)).await
    }

    pub async fn link(&mut self, from_pid: &ExternalPid, to_pid: &ExternalPid) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        let control = ControlMessage::Link {
            from_pid: OwnedTerm::Pid(from_pid.clone()),
            to_pid: OwnedTerm::Pid(to_pid.clone()),
        };

        self.send_control_message(control, None).await
    }

    pub async fn unlink(
        &mut self,
        from_pid: &ExternalPid,
        to_pid: &ExternalPid,
        unlink_id: u64,
    ) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        let control = ControlMessage::UnlinkId {
            id: unlink_id,
            from_pid: OwnedTerm::Pid(from_pid.clone()),
            to_pid: OwnedTerm::Pid(to_pid.clone()),
        };

        self.send_control_message(control, None).await
    }

    pub async fn monitor(
        &mut self,
        from_pid: &ExternalPid,
        to_proc: &ExternalPid,
        reference: &ExternalReference,
    ) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        let control = ControlMessage::MonitorP {
            from_pid: OwnedTerm::Pid(from_pid.clone()),
            to_proc: OwnedTerm::Pid(to_proc.clone()),
            reference: OwnedTerm::Reference(reference.clone()),
        };

        self.send_control_message(control, None).await
    }

    pub async fn demonitor(
        &mut self,
        from_pid: &ExternalPid,
        to_proc: &ExternalPid,
        reference: &ExternalReference,
    ) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        let control = ControlMessage::DemonitorP {
            from_pid: OwnedTerm::Pid(from_pid.clone()),
            to_proc: OwnedTerm::Pid(to_proc.clone()),
            reference: OwnedTerm::Reference(reference.clone()),
        };

        self.send_control_message(control, None).await
    }

    #[doc(hidden)]
    pub fn decode_complete_fragment(
        complete_data: &[u8],
        atom_cache: &mut AtomCache,
    ) -> Result<(ControlMessage, Option<OwnedTerm>)> {
        let (control_term, message) = if complete_data.len() >= 2
            && complete_data[0] == VERSION_TAG
            && complete_data[1] == DIST_HEADER
        {
            decoder::decode_with_atom_cache(complete_data, atom_cache)?
        } else {
            (decoder::decode(complete_data)?, None)
        };

        let control = ControlMessage::from_term(&control_term)?;
        Ok((control, message))
    }

    pub async fn receive_message(&mut self) -> Result<(ControlMessage, Option<OwnedTerm>)> {
        if !self.is_connected() {
            return Err(Error::InvalidState {
                state: self.state(),
            });
        }

        loop {
            let data = self.read_message().await?;

            if data.is_empty() {
                trace!("Received tick (heartbeat), continuing...");
                continue;
            }

            trace!(
                "Decoding message (first 20 bytes): {:02x?}",
                &data[..data.len().min(20)]
            );

            if data.len() >= 2 && data[0] == VERSION_TAG && data[1] == DIST_FRAG_HEADER {
                trace!("DIST_FRAG_HEADER detected");
                let (header, remaining) = decoder::decode_fragment_header(&data)?;
                trace!(
                    "Fragment header: seq={}, frag={}/{}",
                    header.sequence_id, header.fragment_id, header.fragment_id
                );

                let atom_cache_data = if header.num_atom_cache_refs > 0 {
                    Some(remaining[..header.num_atom_cache_refs as usize].to_vec())
                } else {
                    None
                };

                let payload_start = if header.num_atom_cache_refs > 0 {
                    header.num_atom_cache_refs as usize
                } else {
                    0
                };

                if let Some(complete_data) = self.fragment_assembler.start_fragment(
                    header.sequence_id,
                    header.fragment_id,
                    atom_cache_data,
                    remaining[payload_start..].to_vec(),
                ) {
                    trace!("Fragment sequence complete, processing");
                    return Self::decode_complete_fragment(&complete_data, &mut self.atom_cache);
                } else {
                    continue;
                }
            } else if data.len() >= 2 && data[0] == VERSION_TAG && data[1] == DIST_FRAG_CONT {
                trace!("DIST_FRAG_CONT detected");
                let ((sequence_id, fragment_id), remaining) = decoder::decode_fragment_cont(&data)?;
                trace!(
                    "Fragment continuation: seq={}, frag={}",
                    sequence_id, fragment_id
                );

                if let Some(complete_data) = self.fragment_assembler.add_fragment(
                    sequence_id,
                    fragment_id,
                    remaining.to_vec(),
                ) {
                    trace!("Fragment sequence complete, processing");
                    return Self::decode_complete_fragment(&complete_data, &mut self.atom_cache);
                } else {
                    continue;
                }
            }

            let (control_term, message) = if !data.is_empty() && data[0] == PASS_THROUGH {
                trace!("Pass-through message detected");
                let (control, remaining) = decoder::decode_with_trailing(&data[1..])?;
                trace!(
                    "Decoded control term from pass-through message, {} bytes remaining",
                    remaining.len()
                );
                let message = if !remaining.is_empty() {
                    let (msg, _) = decoder::decode_with_trailing(remaining)?;
                    trace!("Decoded message term from pass-through message");
                    Some(msg)
                } else {
                    None
                };
                (control, message)
            } else if data.len() >= 2 && data[0] == VERSION_TAG && data[1] == DIST_HEADER {
                let (control, payload_opt) =
                    decoder::decode_with_atom_cache(&data, &mut self.atom_cache)?;
                (control, payload_opt)
            } else {
                (decoder::decode(&data)?, None)
            };

            let control = ControlMessage::from_term(&control_term)?;

            trace!("Received control message: {:?}", control);

            return Ok((control, message));
        }
    }

    async fn send_control_message(
        &mut self,
        control: ControlMessage,
        message: Option<OwnedTerm>,
    ) -> Result<()> {
        let control_term = control.to_term();

        let mut buf = BytesMut::new();

        let use_pass_through = self
            .negotiated_flags()
            .as_ref()
            .map(|f| !f.has(DistributionFlags::DIST_HDR_ATOM_CACHE))
            .unwrap_or(true);

        if use_pass_through {
            let control_encoded = erltf::encode(&control_term)?;

            if let Some(msg) = message {
                let msg_encoded = erltf::encode(&msg)?;
                let total_len = 1 + control_encoded.len() + msg_encoded.len();
                trace!(
                    "Sending pass-through message: control_len={}, msg_len={}, total_len={}",
                    control_encoded.len(),
                    msg_encoded.len(),
                    total_len
                );

                let stream = self
                    .transport
                    .write_half_mut()
                    .ok_or_else(|| Error::InvalidStateMessage("no active stream".to_string()))?;

                stream.write_u32(total_len as u32).await?;
                stream.write_u8(PASS_THROUGH).await?;
                stream.write_all(&control_encoded).await?;
                stream.write_all(&msg_encoded).await?;
                stream.flush().await?;
            } else {
                let total_len = 1 + control_encoded.len();
                trace!(
                    "Sending pass-through control: control_len={}, total_len={}",
                    control_encoded.len(),
                    total_len
                );

                let stream = self
                    .transport
                    .write_half_mut()
                    .ok_or_else(|| Error::InvalidStateMessage("no active stream".to_string()))?;

                stream.write_u32(total_len as u32).await?;
                stream.write_u8(PASS_THROUGH).await?;
                stream.write_all(&control_encoded).await?;
                stream.flush().await?;
            }

            trace!("Sent control message: {:?}", control);
            return Ok(());
        }

        if let Some(msg) = message {
            let encoded = erltf::encode_with_dist_header_multi(&[&control_term, &msg])?;
            buf.put_u32(encoded.len() as u32);
            buf.put_slice(&encoded);

            trace!("Sending DIST_HEADER message: total_len={}", encoded.len());
            trace!(
                "Encoded bytes (hex, first 100): {:02x?}",
                &encoded[..encoded.len().min(100)]
            );
        } else {
            let encoded = erltf::encode_with_dist_header(&control_term)?;
            buf.put_u32(encoded.len() as u32);
            buf.put_slice(&encoded);

            trace!("Sending DIST_HEADER control: total_len={}", encoded.len());
        }

        let stream = self
            .transport
            .write_half_mut()
            .ok_or_else(|| Error::InvalidStateMessage("no active stream".to_string()))?;

        tokio::time::timeout(self.config.timeout, stream.write_all(&buf))
            .await
            .map_err(|_| Error::Timeout(self.config.timeout))??;

        tokio::time::timeout(self.config.timeout, stream.flush())
            .await
            .map_err(|_| Error::Timeout(self.config.timeout))??;

        trace!("Sent control message: {:?}", control);

        Ok(())
    }

    pub fn take_read_half(&mut self) -> Option<OwnedReadHalf> {
        self.transport.take_read_half()
    }

    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.config.timeout
    }

    pub async fn receive_message_from_read_half(
        read_half: &mut OwnedReadHalf,
        timeout: Duration,
    ) -> Result<(ControlMessage, Option<OwnedTerm>)> {
        loop {
            let len = {
                trace!("Attempting to read message length (4 bytes, distribution protocol)...");
                let mut len_bytes = [0u8; 4];
                tokio::time::timeout(timeout, read_half.read_exact(&mut len_bytes))
                    .await
                    .map_err(|_| Error::Timeout(timeout))??;
                let len = u32::from_be_bytes(len_bytes);
                trace!(
                    "Read message length: {} bytes (raw bytes: {:02x?})",
                    len, len_bytes
                );
                len as usize
            };

            if len == 0 {
                trace!("Received tick (heartbeat), continuing...");
                continue;
            }

            if len > MAX_MESSAGE_SIZE {
                return Err(Error::MessageTooLarge {
                    size: len,
                    max: MAX_MESSAGE_SIZE,
                });
            }

            let mut buf = vec![0u8; len];
            trace!("Reading {} bytes of message data...", len);
            tokio::time::timeout(timeout, read_half.read_exact(&mut buf))
                .await
                .map_err(|_| Error::Timeout(timeout))??;

            trace!("Read message data (hex): {:02x?}", buf);

            if buf.is_empty() {
                return Err(Error::InvalidStateMessage(
                    "Empty message received".to_string(),
                ));
            }

            let pass_through_marker = buf[0];
            trace!("Pass-through marker: {}", pass_through_marker);

            if pass_through_marker != PASS_THROUGH {
                return Err(Error::Protocol(format!(
                    "Expected pass-through marker {}, got {}",
                    PASS_THROUGH, pass_through_marker
                )));
            }

            let control_and_payload = &buf[1..];
            trace!(
                "Decoding control and payload from {} bytes",
                control_and_payload.len()
            );

            let (control_term, remaining) = decoder::decode_with_trailing(control_and_payload)?;
            trace!("Decoded control term: {:?}", control_term);
            trace!("Remaining bytes after control: {}", remaining.len());

            let control_msg = ControlMessage::from_term(&control_term)?;
            trace!("Parsed control message: {:?}", control_msg);

            let payload = if !remaining.is_empty() {
                trace!("Decoding payload from {} bytes", remaining.len());
                let (payload_term, _) = decoder::decode_with_trailing(remaining)?;
                trace!("Decoded payload: {:?}", payload_term);
                Some(payload_term)
            } else {
                trace!("No payload present");
                None
            };

            return Ok((control_msg, payload));
        }
    }
}
