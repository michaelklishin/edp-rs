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

use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::trace;

const MAX_MESSAGE_SIZE: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameMode {
    Handshake,
    Distribution,
}

impl FrameMode {
    pub fn length_prefix_size(&self) -> usize {
        match self {
            FrameMode::Handshake => 2,
            FrameMode::Distribution => 4,
        }
    }
}

pub struct MessageFramer {
    mode: FrameMode,
}

impl MessageFramer {
    pub fn new(mode: FrameMode) -> Self {
        Self { mode }
    }

    pub fn set_mode(&mut self, mode: FrameMode) {
        self.mode = mode;
    }

    pub fn frame_message(&self, data: &[u8]) -> Vec<u8> {
        let mut buf = match self.mode {
            FrameMode::Handshake => {
                let len = data.len() as u16;
                let mut b = BytesMut::with_capacity(2 + data.len());
                b.put_u16(len);
                b
            }
            FrameMode::Distribution => {
                let len = data.len() as u32;
                let mut b = BytesMut::with_capacity(4 + data.len());
                b.put_u32(len);
                b
            }
        };
        buf.put_slice(data);
        buf.to_vec()
    }

    pub async fn write_framed<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
        data: &[u8],
    ) -> std::io::Result<()> {
        trace!(
            "Writing {} bytes in {:?} mode: {:02x?}",
            data.len(),
            self.mode,
            data
        );

        match self.mode {
            FrameMode::Handshake => {
                let len = data.len() as u16;
                writer.write_u16(len).await?;
            }
            FrameMode::Distribution => {
                let len = data.len() as u32;
                writer.write_u32(len).await?;
            }
        }
        writer.write_all(data).await?;
        writer.flush().await?;
        Ok(())
    }
}

pub struct MessageDeframer {
    mode: FrameMode,
}

impl MessageDeframer {
    pub fn new(mode: FrameMode) -> Self {
        Self { mode }
    }

    pub fn set_mode(&mut self, mode: FrameMode) {
        self.mode = mode;
    }

    pub async fn read_framed<R: AsyncRead + Unpin>(
        &self,
        reader: &mut R,
    ) -> std::io::Result<Vec<u8>> {
        let len = match self.mode {
            FrameMode::Handshake => {
                trace!("Reading message length (2 bytes, handshake mode)");
                let len = reader.read_u16().await?;
                trace!("Read length: {} bytes", len);
                len as usize
            }
            FrameMode::Distribution => {
                trace!("Reading message length (4 bytes, distribution mode)");
                let mut len_bytes = [0u8; 4];
                reader.read_exact(&mut len_bytes).await?;
                let len = u32::from_be_bytes(len_bytes);
                trace!("Read length: {} bytes (raw: {:02x?})", len, len_bytes);
                len as usize
            }
        };

        if len == 0 {
            trace!("Received 0-byte message (heartbeat/tick)");
            return Ok(Vec::new());
        }

        if len > MAX_MESSAGE_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Message too large: {} bytes (max: {})",
                    len, MAX_MESSAGE_SIZE
                ),
            ));
        }

        let mut buf = vec![0u8; len];
        trace!("Reading {} bytes of message data", len);
        reader.read_exact(&mut buf).await?;
        trace!("Read message data (hex): {:02x?}", buf);

        Ok(buf)
    }
}
