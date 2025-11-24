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

use edp_client::framing::{FrameMode, MessageDeframer, MessageFramer};
use std::io::Cursor;

#[test]
fn test_handshake_framing() {
    let framer = MessageFramer::new(FrameMode::Handshake);
    let data = b"hello";
    let framed = framer.frame_message(data);

    assert_eq!(framed.len(), 2 + data.len());
    assert_eq!(&framed[0..2], &[0, 5]);
    assert_eq!(&framed[2..], data);
}

#[test]
fn test_distribution_framing() {
    let framer = MessageFramer::new(FrameMode::Distribution);
    let data = b"hello world";
    let framed = framer.frame_message(data);

    assert_eq!(framed.len(), 4 + data.len());
    assert_eq!(&framed[0..4], &[0, 0, 0, 11]);
    assert_eq!(&framed[4..], data);
}

#[tokio::test]
async fn test_roundtrip_handshake() {
    let framer = MessageFramer::new(FrameMode::Handshake);
    let deframer = MessageDeframer::new(FrameMode::Handshake);

    let data = b"test message";
    let framed = framer.frame_message(data);

    let mut cursor = Cursor::new(framed);
    let result = deframer.read_framed(&mut cursor).await.unwrap();

    assert_eq!(result, data);
}

#[tokio::test]
async fn test_roundtrip_distribution() {
    let framer = MessageFramer::new(FrameMode::Distribution);
    let deframer = MessageDeframer::new(FrameMode::Distribution);

    let data = b"another test";
    let framed = framer.frame_message(data);

    let mut cursor = Cursor::new(framed);
    let result = deframer.read_framed(&mut cursor).await.unwrap();

    assert_eq!(result, data);
}

#[tokio::test]
async fn test_empty_message() {
    let deframer = MessageDeframer::new(FrameMode::Distribution);
    let framed = vec![0, 0, 0, 0];

    let mut cursor = Cursor::new(framed);
    let result = deframer.read_framed(&mut cursor).await.unwrap();

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_message_too_large_error() {
    let deframer = MessageDeframer::new(FrameMode::Distribution);
    let too_large = 256 * 1024 * 1024 + 1;
    let mut framed = vec![0u8; 4];
    framed[0..4].copy_from_slice(&(too_large as u32).to_be_bytes());

    let mut cursor = Cursor::new(framed);
    let result = deframer.read_framed(&mut cursor).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("Message too large"));
}
