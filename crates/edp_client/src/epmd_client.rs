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

//! An EPMD (Erlang Port Mapper Daemon) protocol client.

use crate::errors::{Error, Result};
use bytes::{BufMut, BytesMut};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Default EPMD port
pub const EPMD_PORT: u16 = 4369;

/// Default connection timeout
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// EPMD message types
const ALIVE2_REQ: u8 = 120;
const ALIVE2_RESP: u8 = 121;
const ALIVE2_X_RESP: u8 = 118;
const PORT2_REQ: u8 = 122;
const PORT2_RESP: u8 = 119;
const NAMES_REQ: u8 = 110;
const DUMP_REQ: u8 = 100;
const KILL_REQ: u8 = 107;

/// Node types for EPMD registration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeType {
    /// Normal Erlang node
    Normal = 77, // 'M'
    /// Hidden node (not in global name registry)
    Hidden = 72, // 'H'
    /// R3 compatibility
    R3Hidden = 104, // 'h'
}

/// Protocol type for EPMD registration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Protocol {
    /// TCP/IP based protocol
    Tcp = 0,
}

/// Node information from EPMD PORT2_RESP
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    pub port: u16,
    pub node_type: NodeType,
    pub protocol: Protocol,
    pub highest_version: u16,
    pub lowest_version: u16,
    pub node_name: String,
    pub extra: Vec<u8>,
}

/// EPMD client for node registration and lookup
pub struct EpmdClient {
    host: String,
    port: u16,
    timeout: Duration,
}

impl EpmdClient {
    /// Create a new EPMD client
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: EPMD_PORT,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Create an EPMD client with custom port
    pub fn with_port(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Set connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn connect(&self) -> Result<TcpStream> {
        let addr = format!("{}:{}", self.host, self.port);
        tokio::time::timeout(self.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| Error::Timeout(self.timeout))?
            .map_err(|e| {
                Error::EpmdProtocol(format!("Failed to connect to EPMD at {}: {}", addr, e))
            })
    }

    /// Lookup a node's port by name
    pub async fn lookup_node(&self, node_name: &str) -> Result<NodeInfo> {
        let mut stream = self.connect().await?;

        let mut buf = BytesMut::new();
        buf.put_u16(node_name.len() as u16 + 1);
        buf.put_u8(PORT2_REQ);
        buf.put_slice(node_name.as_bytes());

        stream.write_all(&buf).await?;
        stream.flush().await?;

        let response_type = stream.read_u8().await?;

        match response_type {
            PORT2_RESP => {
                let result = stream.read_u8().await?;
                if result != 0 {
                    return Err(Error::EpmdLookup {
                        node: node_name.to_string(),
                        reason: "Node not found".to_string(),
                    });
                }

                let port = stream.read_u16().await?;
                let node_type = match stream.read_u8().await? {
                    77 => NodeType::Normal,
                    72 => NodeType::Hidden,
                    104 => NodeType::R3Hidden,
                    other => {
                        return Err(Error::EpmdProtocol(format!("Unknown node type: {}", other)));
                    }
                };

                let protocol = match stream.read_u8().await? {
                    0 => Protocol::Tcp,
                    other => {
                        return Err(Error::EpmdProtocol(format!("Unknown protocol: {}", other)));
                    }
                };

                let highest_version = stream.read_u16().await?;
                let lowest_version = stream.read_u16().await?;

                let nlen = stream.read_u16().await?;
                if nlen > 255 {
                    return Err(Error::EpmdProtocol(format!(
                        "Node name too long: {} bytes",
                        nlen
                    )));
                }
                let mut name_buf = vec![0u8; nlen as usize];
                stream.read_exact(&mut name_buf).await?;
                let node_name = String::from_utf8(name_buf)
                    .map_err(|_| Error::EpmdProtocol("Invalid UTF-8 in node name".to_string()))?;

                let elen = stream.read_u16().await?;
                if elen > 4096 {
                    return Err(Error::EpmdProtocol(format!(
                        "Extra data too long: {} bytes",
                        elen
                    )));
                }
                let mut extra = vec![0u8; elen as usize];
                if elen > 0 {
                    stream.read_exact(&mut extra).await?;
                }

                Ok(NodeInfo {
                    port,
                    node_type,
                    protocol,
                    highest_version,
                    lowest_version,
                    node_name,
                    extra,
                })
            }
            other => Err(Error::EpmdProtocol(format!(
                "Unexpected response type: {}",
                other
            ))),
        }
    }

    /// Register a node with EPMD (ALIVE2_REQ)
    pub async fn register_node(
        &self,
        port: u16,
        node_name: &str,
        node_type: NodeType,
        highest_version: u16,
        lowest_version: u16,
        extra: &[u8],
    ) -> Result<u32> {
        let mut stream = self.connect().await?;

        let name_bytes = node_name.as_bytes();
        let total_len = 1 + 2 + 1 + 1 + 2 + 2 + 2 + name_bytes.len() + 2 + extra.len();

        let mut buf = BytesMut::new();
        buf.put_u16(total_len as u16);
        buf.put_u8(ALIVE2_REQ);
        buf.put_u16(port);
        buf.put_u8(node_type as u8);
        buf.put_u8(Protocol::Tcp as u8);
        buf.put_u16(highest_version);
        buf.put_u16(lowest_version);
        buf.put_u16(name_bytes.len() as u16);
        buf.put_slice(name_bytes);
        buf.put_u16(extra.len() as u16);
        buf.put_slice(extra);

        stream.write_all(&buf).await?;
        stream.flush().await?;

        let response_type = stream.read_u8().await?;

        match response_type {
            ALIVE2_RESP => {
                let result = stream.read_u8().await?;
                if result != 0 {
                    return Err(Error::EpmdRegistration {
                        reason: format!("EPMD returned error code: {}", result),
                    });
                }

                let creation = stream.read_u16().await? as u32;
                Ok(creation)
            }
            ALIVE2_X_RESP => {
                let result = stream.read_u8().await?;
                if result != 0 {
                    return Err(Error::EpmdRegistration {
                        reason: format!("EPMD returned error code: {}", result),
                    });
                }

                let creation = stream.read_u32().await?;
                Ok(creation)
            }
            other => Err(Error::EpmdProtocol(format!(
                "Unexpected response type: {}",
                other
            ))),
        }
    }

    /// Query all registered nodes (NAMES_REQ)
    pub async fn list_nodes(&self) -> Result<String> {
        let mut stream = self.connect().await?;

        let mut buf = BytesMut::new();
        buf.put_u16(1);
        buf.put_u8(NAMES_REQ);

        stream.write_all(&buf).await?;
        stream.flush().await?;

        let _epmd_port = stream.read_u32().await?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;

        String::from_utf8(response)
            .map_err(|_| Error::EpmdProtocol("Invalid UTF-8 in NAMES response".to_string()))
    }

    /// Dump all registered nodes with details (DUMP_REQ)
    pub async fn dump_nodes(&self) -> Result<String> {
        let mut stream = self.connect().await?;

        let mut buf = BytesMut::new();
        buf.put_u16(1);
        buf.put_u8(DUMP_REQ);

        stream.write_all(&buf).await?;
        stream.flush().await?;

        let _epmd_port = stream.read_u32().await?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;

        String::from_utf8(response)
            .map_err(|_| Error::EpmdProtocol("Invalid UTF-8 in DUMP response".to_string()))
    }

    /// Kill the EPMD daemon (KILL_REQ)
    pub async fn kill_daemon(&self) -> Result<String> {
        let mut stream = self.connect().await?;

        let mut buf = BytesMut::new();
        buf.put_u16(1);
        buf.put_u8(KILL_REQ);

        stream.write_all(&buf).await?;
        stream.flush().await?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await?;

        String::from_utf8(response)
            .map_err(|_| Error::EpmdProtocol("Invalid UTF-8 in KILL response".to_string()))
    }
}
