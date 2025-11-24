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

//! Distribution protocol handshake messages and state machine.

use crate::digest;
use crate::errors::{Error, Result};
use crate::flags::DistributionFlags;
use bytes::{Buf, BufMut, BytesMut};
use std::fmt;

/// Handshake protocol version (version 6 was introduced in Erlang/OTP 23.0)
pub const PROTOCOL_VERSION: u16 = 6;

/// Old protocol version 5 (for compatibility)
pub const PROTOCOL_VERSION_5: u16 = 5;

/// Handshake message tags
const HANDSHAKE_TAG_N: u8 = b'N'; // New format (v6)
const HANDSHAKE_TAG_N_OLD: u8 = b'n'; // Old format (v5)
const HANDSHAKE_TAG_S: u8 = b's';
const HANDSHAKE_TAG_A: u8 = b'a';

/// Handshake status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok = 0,
    OkSimultaneous = 1,
    Nok = 2,
    NotAllowed = 3,
    Alive = 4,
}

impl Status {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Status::Ok),
            1 => Some(Status::OkSimultaneous),
            2 => Some(Status::Nok),
            3 => Some(Status::NotAllowed),
            4 => Some(Status::Alive),
            _ => None,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Status::Ok | Status::OkSimultaneous)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "ok"),
            Status::OkSimultaneous => write!(f, "ok_simultaneous"),
            Status::Nok => write!(f, "nok"),
            Status::NotAllowed => write!(f, "not_allowed"),
            Status::Alive => write!(f, "alive"),
        }
    }
}

/// Send Name message (tag: 'N' for protocol v6)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendName {
    pub flags: DistributionFlags,
    pub creation: u32,
    pub name: String,
}

impl SendName {
    pub fn new(flags: DistributionFlags, creation: u32, name: impl Into<String>) -> Self {
        Self {
            flags,
            creation,
            name: name.into(),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let name_bytes = self.name.as_bytes();
        if name_bytes.len() > 255 {
            return Err(Error::NodeNameTooLong {
                size: name_bytes.len(),
                max: 255,
            });
        }

        let mut buf = BytesMut::new();

        let message_len = 1 + 8 + 4 + 2 + name_bytes.len();
        buf.put_u16(message_len as u16);
        buf.put_u8(HANDSHAKE_TAG_N);
        buf.put_u64(self.flags.as_u64());
        buf.put_u32(self.creation);
        buf.put_u16(name_bytes.len() as u16);
        buf.put_slice(name_bytes);

        Ok(buf.to_vec())
    }

    pub fn encode_old(&self) -> Result<Vec<u8>> {
        let name_bytes = self.name.as_bytes();
        if name_bytes.len() > 255 {
            return Err(Error::NodeNameTooLong {
                size: name_bytes.len(),
                max: 255,
            });
        }

        let mut buf = BytesMut::new();

        let message_len = 1 + 2 + 4 + name_bytes.len();
        buf.put_u16(message_len as u16);
        buf.put_u8(HANDSHAKE_TAG_N_OLD);
        buf.put_u16(PROTOCOL_VERSION_5);
        buf.put_u32(self.flags.as_u64() as u32);
        buf.put_slice(name_bytes);

        Ok(buf.to_vec())
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buf = data;

        if buf.remaining() < 1 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for tag".to_string(),
            ));
        }

        let tag = buf.get_u8();
        if tag != HANDSHAKE_TAG_N {
            return Err(Error::InvalidHandshakeMessage(format!(
                "Expected tag 'N' ({}), got {}",
                HANDSHAKE_TAG_N, tag
            )));
        }

        if buf.remaining() < 8 + 4 + 2 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for flags, creation, and name length".to_string(),
            ));
        }

        let flags = DistributionFlags::new(buf.get_u64());
        let creation = buf.get_u32();
        let name_len = buf.get_u16() as usize;

        if buf.remaining() < name_len {
            return Err(Error::InvalidHandshakeMessage(format!(
                "Insufficient data for name: expected {} bytes, got {}",
                name_len,
                buf.remaining()
            )));
        }

        let name_bytes = &buf[..name_len];
        let name = std::str::from_utf8(name_bytes)
            .map_err(|_| Error::InvalidHandshakeMessage("Invalid UTF-8 in node name".to_string()))?
            .to_owned();

        Ok(Self {
            flags,
            creation,
            name,
        })
    }
}

/// Status message (tag: 's')
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMessage {
    pub status: Status,
}

impl StatusMessage {
    pub fn new(status: Status) -> Self {
        Self { status }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u16(3);
        buf.put_u8(HANDSHAKE_TAG_S);
        buf.put_u16(self.status as u16);
        buf.to_vec()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buf = data;

        if buf.remaining() < 1 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for tag".to_string(),
            ));
        }

        let tag = buf.get_u8();
        if tag != HANDSHAKE_TAG_S {
            return Err(Error::InvalidHandshakeMessage(format!(
                "Expected tag 's' ({}), got {}",
                HANDSHAKE_TAG_S, tag
            )));
        }

        let status_str = std::str::from_utf8(buf)
            .map_err(|_| Error::InvalidHandshakeMessage("Invalid UTF-8 in status".to_string()))?;

        let status = match status_str {
            "ok" => Status::Ok,
            "ok_simultaneous" => Status::OkSimultaneous,
            "nok" => Status::Nok,
            "not_allowed" => Status::NotAllowed,
            "alive" => Status::Alive,
            _ => {
                return Err(Error::InvalidHandshakeMessage(format!(
                    "Unknown status: {}",
                    status_str
                )));
            }
        };

        Ok(Self { status })
    }
}

/// Challenge message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    pub flags: DistributionFlags,
    pub challenge: u32,
    pub creation: u32,
    pub name: String,
}

impl Challenge {
    pub fn new(
        flags: DistributionFlags,
        challenge: u32,
        creation: u32,
        name: impl Into<String>,
    ) -> Self {
        Self {
            flags,
            challenge,
            creation,
            name: name.into(),
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let name_bytes = self.name.as_bytes();
        if name_bytes.len() > 255 {
            return Err(Error::NodeNameTooLong {
                size: name_bytes.len(),
                max: 255,
            });
        }

        let mut buf = BytesMut::new();

        let message_len = 1 + 8 + 4 + 4 + 2 + name_bytes.len();
        buf.put_u16(message_len as u16);
        buf.put_u8(HANDSHAKE_TAG_N);
        buf.put_u64(self.flags.as_u64());
        buf.put_u32(self.challenge);
        buf.put_u32(self.creation);
        buf.put_u16(name_bytes.len() as u16);
        buf.put_slice(name_bytes);

        Ok(buf.to_vec())
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buf = data;

        if buf.remaining() < 1 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for tag".to_string(),
            ));
        }

        let tag = buf.get_u8();
        if tag != HANDSHAKE_TAG_N {
            return Err(Error::InvalidHandshakeMessage(format!(
                "Expected tag 'N', got {}",
                tag
            )));
        }

        if buf.remaining() < 8 + 4 + 4 + 2 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for flags, challenge, creation, name length".to_string(),
            ));
        }

        let flags = DistributionFlags::new(buf.get_u64());
        let challenge = buf.get_u32();
        let creation = buf.get_u32();
        let name_len = buf.get_u16() as usize;

        if buf.remaining() < name_len {
            return Err(Error::InvalidHandshakeMessage(format!(
                "Insufficient data for name: expected {} bytes, got {}",
                name_len,
                buf.remaining()
            )));
        }

        let name_bytes = &buf[..name_len];
        let name = std::str::from_utf8(name_bytes)
            .map_err(|_| Error::InvalidHandshakeMessage("Invalid UTF-8 in node name".to_string()))?
            .to_owned();

        Ok(Self {
            flags,
            challenge,
            creation,
            name,
        })
    }
}

/// Challenge Reply message (tag: 'r')
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeReply {
    pub challenge: u32,
    pub digest: [u8; 16],
}

impl ChallengeReply {
    pub fn new(our_challenge: u32, their_challenge: u32, cookie: &str) -> Self {
        let digest = digest::compute_digest(their_challenge, cookie);
        Self {
            challenge: our_challenge,
            digest,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u16(21);
        buf.put_u8(b'r');
        buf.put_u32(self.challenge);
        buf.put_slice(&self.digest);
        buf.to_vec()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buf = data;

        if buf.remaining() < 1 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for tag".to_string(),
            ));
        }

        let tag = buf.get_u8();
        if tag != b'r' {
            return Err(Error::InvalidHandshakeMessage(format!(
                "Expected tag 'r', got {}",
                tag
            )));
        }

        if buf.remaining() < 4 + 16 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for challenge and digest".to_string(),
            ));
        }

        let challenge = buf.get_u32();
        let mut digest = [0u8; 16];
        buf.copy_to_slice(&mut digest);

        Ok(Self { challenge, digest })
    }

    pub fn verify(&self, their_challenge: u32, cookie: &str) -> bool {
        let expected_digest = digest::compute_digest(their_challenge, cookie);
        self.digest == expected_digest
    }
}

/// Challenge Acknowledgment message (tag: 'a')
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeAck {
    pub digest: [u8; 16],
}

impl ChallengeAck {
    pub fn new(challenge: u32, cookie: &str) -> Self {
        let digest = digest::compute_digest(challenge, cookie);
        Self { digest }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u16(17);
        buf.put_u8(HANDSHAKE_TAG_A);
        buf.put_slice(&self.digest);
        buf.to_vec()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buf = data;

        if buf.remaining() < 1 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for tag".to_string(),
            ));
        }

        let tag = buf.get_u8();
        if tag != HANDSHAKE_TAG_A {
            return Err(Error::InvalidHandshakeMessage(format!(
                "Expected tag 'a', got {}",
                tag
            )));
        }

        if buf.remaining() < 16 {
            return Err(Error::InvalidHandshakeMessage(
                "Insufficient data for digest".to_string(),
            ));
        }

        let mut digest = [0u8; 16];
        buf.copy_to_slice(&mut digest);

        Ok(Self { digest })
    }

    pub fn verify(&self, challenge: u32, cookie: &str) -> bool {
        let expected_digest = digest::compute_digest(challenge, cookie);
        self.digest == expected_digest
    }
}
