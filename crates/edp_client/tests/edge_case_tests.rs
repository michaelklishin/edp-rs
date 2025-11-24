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

use edp_client::Error;
use edp_client::control::ControlMessage;
use edp_client::handshake::{Challenge, ChallengeAck, ChallengeReply, SendName};
use erltf::OwnedTerm;

//
// Node Name Length Limits
//

#[test]
fn test_send_name_max_length() {
    let name = "a".repeat(255);
    let flags = edp_client::DistributionFlags::default();
    let send_name = SendName::new(flags, 1, name);
    assert!(send_name.encode().is_ok());
}

#[test]
fn test_send_name_too_long() {
    let name = "a".repeat(256);
    let flags = edp_client::DistributionFlags::default();
    let send_name = SendName::new(flags, 1, name);
    let result = send_name.encode();
    assert!(matches!(result, Err(Error::NodeNameTooLong { .. })));
}

#[test]
fn test_challenge_max_length_name() {
    let name = "a".repeat(255);
    let flags = edp_client::DistributionFlags::default();
    let challenge = Challenge::new(flags, 12345, 1, name);
    assert!(challenge.encode().is_ok());
}

#[test]
fn test_challenge_too_long_name() {
    let name = "a".repeat(256);
    let flags = edp_client::DistributionFlags::default();
    let challenge = Challenge::new(flags, 12345, 1, name);
    let result = challenge.encode();
    assert!(matches!(result, Err(Error::NodeNameTooLong { .. })));
}

//
// Handshake Boundary Values
//

#[test]
fn test_challenge_reply_with_zero_challenge() {
    let reply = ChallengeReply::new(0, 0, "cookie");
    assert!(reply.verify(0, "cookie"));
    assert!(!reply.verify(1, "cookie"));
}

#[test]
fn test_challenge_reply_with_max_challenge() {
    let reply = ChallengeReply::new(u32::MAX, u32::MAX, "cookie");
    assert!(reply.verify(u32::MAX, "cookie"));
    assert!(!reply.verify(u32::MAX - 1, "cookie"));
}

#[test]
fn test_challenge_ack_with_boundary_values() {
    let ack_zero = ChallengeAck::new(0, "cookie");
    assert!(ack_zero.verify(0, "cookie"));

    let ack_max = ChallengeAck::new(u32::MAX, "cookie");
    assert!(ack_max.verify(u32::MAX, "cookie"));
}

//
// Control Message Validation
//

#[test]
fn test_control_message_type_out_of_range() {
    let term = OwnedTerm::Tuple(vec![OwnedTerm::Integer(256)]);
    let result = ControlMessage::from_term(&term);
    assert!(matches!(result, Err(Error::InvalidControlMessage(_))));
}

#[test]
fn test_control_message_negative_type() {
    let term = OwnedTerm::Tuple(vec![OwnedTerm::Integer(-1)]);
    let result = ControlMessage::from_term(&term);
    assert!(matches!(result, Err(Error::InvalidControlMessage(_))));
}

#[test]
fn test_control_message_unlink_id_negative() {
    let term = OwnedTerm::Tuple(vec![
        OwnedTerm::Integer(35),
        OwnedTerm::Integer(-1),
        OwnedTerm::Tuple(vec![]),
        OwnedTerm::Tuple(vec![]),
    ]);
    let result = ControlMessage::from_term(&term);
    assert!(matches!(result, Err(Error::InvalidControlMessage(_))));
}

#[test]
fn test_control_message_unlink_id_ack_negative() {
    let term = OwnedTerm::Tuple(vec![
        OwnedTerm::Integer(36),
        OwnedTerm::Integer(-1),
        OwnedTerm::Tuple(vec![]),
        OwnedTerm::Tuple(vec![]),
    ]);
    let result = ControlMessage::from_term(&term);
    assert!(matches!(result, Err(Error::InvalidControlMessage(_))));
}

#[test]
fn test_control_message_empty_tuple() {
    let term = OwnedTerm::Tuple(vec![]);
    let result = ControlMessage::from_term(&term);
    assert!(matches!(result, Err(Error::InvalidControlMessage(_))));
}

#[test]
fn test_control_message_not_a_tuple() {
    let term = OwnedTerm::Integer(42);
    let result = ControlMessage::from_term(&term);
    assert!(matches!(result, Err(Error::InvalidControlMessage(_))));
}

//
// Handshake Message Decoding Errors
//

#[test]
fn test_send_name_invalid_handshake_tag() {
    let mut data = vec![0, 15];
    data.push(b'X');
    data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
    data.extend_from_slice(&[0, 0, 0, 1]);
    data.extend_from_slice(&[0, 4]);
    data.extend_from_slice(b"test");

    let result = SendName::decode(&data);
    assert!(matches!(result, Err(Error::InvalidHandshakeMessage(_))));
}

#[test]
fn test_send_name_truncated_message() {
    let data = vec![b'N', 0, 0];
    let result = SendName::decode(&data);
    assert!(matches!(result, Err(Error::InvalidHandshakeMessage(_))));
}

#[test]
fn test_send_name_insufficient_data_for_name() {
    let mut data = vec![b'N'];
    data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
    data.extend_from_slice(&[0, 0, 0, 1]);
    data.extend_from_slice(&[0, 10]);
    data.extend_from_slice(b"test");

    let result = SendName::decode(&data);
    assert!(matches!(result, Err(Error::InvalidHandshakeMessage(_))));
}

#[test]
fn test_challenge_decode_invalid_utf8() {
    let mut data = vec![b'N'];
    data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
    data.extend_from_slice(&[0, 0, 0, 42]);
    data.extend_from_slice(&[0, 0, 0, 1]);
    data.extend_from_slice(&[0, 4]);
    data.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]);

    let result = Challenge::decode(&data);
    assert!(matches!(result, Err(Error::InvalidHandshakeMessage(_))));
}
