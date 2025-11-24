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

use crate::digest;
use crate::errors::{Error, Result};
use crate::flags::DistributionFlags;
use crate::handshake::{Challenge, ChallengeAck, ChallengeReply, SendName, StatusMessage};
use crate::types::Creation;
use bytes::{BufMut, BytesMut};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    SendingName,
    AwaitingStatus,
    AwaitingChallenge,
    SendingChallengeReply,
    AwaitingChallengeAck,
    Connected,
    Failed,
}

impl ConnectionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionState::Disconnected => "disconnected",
            ConnectionState::Connecting => "connecting",
            ConnectionState::SendingName => "sending_name",
            ConnectionState::AwaitingStatus => "awaiting_status",
            ConnectionState::AwaitingChallenge => "awaiting_challenge",
            ConnectionState::SendingChallengeReply => "sending_challenge_reply",
            ConnectionState::AwaitingChallengeAck => "awaiting_challenge_ack",
            ConnectionState::Connected => "connected",
            ConnectionState::Failed => "failed",
        }
    }
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct HandshakeStateMachine {
    state: ConnectionState,
    local_node_name: String,
    #[allow(dead_code)]
    remote_node_name: String,
    cookie: String,
    flags: DistributionFlags,
    creation: Creation,
    our_challenge: Option<u32>,
    their_challenge: Option<u32>,
    negotiated_flags: Option<DistributionFlags>,
}

impl HandshakeStateMachine {
    pub fn new<C: Into<Creation>>(
        local_node_name: String,
        remote_node_name: String,
        cookie: String,
        flags: DistributionFlags,
        creation: C,
    ) -> Self {
        Self {
            state: ConnectionState::Disconnected,
            local_node_name,
            remote_node_name,
            cookie,
            flags,
            creation: creation.into(),
            our_challenge: None,
            their_challenge: None,
            negotiated_flags: None,
        }
    }

    #[must_use]
    pub fn state(&self) -> ConnectionState {
        self.state
    }

    #[must_use]
    pub fn negotiated_flags(&self) -> Option<DistributionFlags> {
        self.negotiated_flags
    }

    pub fn begin_connect(&mut self) -> Result<()> {
        if self.state != ConnectionState::Disconnected {
            return Err(Error::InvalidStateTransition {
                from: self.state,
                to: ConnectionState::Connecting,
            });
        }
        self.state = ConnectionState::Connecting;
        Ok(())
    }

    pub fn prepare_send_name(&mut self) -> Result<Vec<u8>> {
        self.state = ConnectionState::SendingName;
        let send_name = SendName::new(self.flags, self.creation.0, &self.local_node_name);
        let data = send_name.encode_old()?;
        self.state = ConnectionState::AwaitingStatus;
        Ok(data)
    }

    pub fn handle_status(&mut self, data: &[u8]) -> Result<()> {
        let status_msg = StatusMessage::decode(data)?;
        if !status_msg.status.is_ok() {
            return Err(Error::ConnectionRefused {
                reason: format!("Status: {}", status_msg.status),
            });
        }
        Ok(())
    }

    pub fn prepare_complement(&mut self) -> Result<Vec<u8>> {
        let flags_u64 = self.flags.as_u64();
        let high_flags = (flags_u64 >> 32) as u32;

        let mut buf = BytesMut::new();
        buf.put_u16(9);
        buf.put_u8(b'c');
        buf.put_u32(high_flags);
        buf.put_u32(self.creation.0);
        Ok(buf.to_vec())
    }

    pub fn handle_challenge(&mut self, data: &[u8]) -> Result<()> {
        self.state = ConnectionState::AwaitingChallenge;
        let challenge = Challenge::decode(data)?;

        self.negotiated_flags = Some(DistributionFlags::new(
            challenge.flags.as_u64() & self.flags.as_u64(),
        ));

        self.their_challenge = Some(challenge.challenge);
        self.our_challenge = Some(digest::generate_challenge());
        Ok(())
    }

    pub fn prepare_challenge_reply(&mut self) -> Result<Vec<u8>> {
        self.state = ConnectionState::SendingChallengeReply;

        let our_challenge = self
            .our_challenge
            .ok_or_else(|| Error::InvalidStateMessage("no our_challenge set".to_string()))?;

        let their_challenge = self
            .their_challenge
            .ok_or_else(|| Error::InvalidStateMessage("no their_challenge set".to_string()))?;

        let reply = ChallengeReply::new(our_challenge, their_challenge, &self.cookie);
        let data = reply.encode();
        self.state = ConnectionState::AwaitingChallengeAck;
        Ok(data)
    }

    pub fn handle_challenge_ack(&mut self, data: &[u8]) -> Result<()> {
        let ack = ChallengeAck::decode(data)?;

        let our_challenge = self
            .our_challenge
            .ok_or_else(|| Error::InvalidStateMessage("no our_challenge set".to_string()))?;

        if !ack.verify(our_challenge, &self.cookie) {
            return Err(Error::AuthenticationFailed);
        }

        self.state = ConnectionState::Connected;
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
        self.our_challenge = None;
        self.their_challenge = None;
        self.negotiated_flags = None;
    }
}
