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
use crate::framing::{FrameMode, MessageDeframer, MessageFramer};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub struct FramedTransport {
    read_half: Option<OwnedReadHalf>,
    write_half: Option<OwnedWriteHalf>,
    framer: MessageFramer,
    deframer: MessageDeframer,
    timeout: Duration,
}

impl FramedTransport {
    pub fn new(timeout: Duration) -> Self {
        Self {
            read_half: None,
            write_half: None,
            framer: MessageFramer::new(FrameMode::Handshake),
            deframer: MessageDeframer::new(FrameMode::Handshake),
            timeout,
        }
    }

    pub fn connect(&mut self, stream: TcpStream) {
        let (read_half, write_half) = stream.into_split();
        self.read_half = Some(read_half);
        self.write_half = Some(write_half);
    }

    pub fn set_frame_mode(&mut self, mode: FrameMode) {
        self.framer.set_mode(mode);
        self.deframer.set_mode(mode);
    }

    pub async fn read(&mut self) -> Result<Vec<u8>> {
        let stream = self
            .read_half
            .as_mut()
            .ok_or_else(|| Error::InvalidStateMessage("no active stream".to_string()))?;

        tokio::time::timeout(self.timeout, self.deframer.read_framed(stream))
            .await
            .map_err(|_| Error::Timeout(self.timeout))?
            .map_err(Error::Io)
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        let stream = self
            .write_half
            .as_mut()
            .ok_or_else(|| Error::InvalidStateMessage("no active stream".to_string()))?;

        tokio::time::timeout(self.timeout, self.framer.write_framed(stream, data))
            .await
            .map_err(|_| Error::Timeout(self.timeout))?
            .map_err(Error::Io)
    }

    pub fn close(&mut self) {
        self.read_half = None;
        self.write_half = None;
    }

    pub fn is_connected(&self) -> bool {
        self.read_half.is_some() && self.write_half.is_some()
    }

    pub fn write_half_mut(&mut self) -> Option<&mut OwnedWriteHalf> {
        self.write_half.as_mut()
    }

    pub fn take_read_half(&mut self) -> Option<OwnedReadHalf> {
        self.read_half.take()
    }

    pub async fn write_raw(&mut self, data: &[u8]) -> Result<()> {
        let stream = self
            .write_half
            .as_mut()
            .ok_or_else(|| Error::InvalidStateMessage("no active stream".to_string()))?;

        tokio::time::timeout(self.timeout, async {
            stream.write_all(data).await?;
            stream.flush().await
        })
        .await
        .map_err(|_| Error::Timeout(self.timeout))?
        .map_err(Error::Io)
    }
}
