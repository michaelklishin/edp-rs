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

use edp_client::flags::DistributionFlags;
use edp_client::handshake::{ChallengeAck, ChallengeReply, SendName, Status};

//
// SendName Message
//

#[test]
fn test_send_name_encode_decode() {
    let original = SendName::new(DistributionFlags::default_otp26(), 123, "test@localhost");

    let encoded = original.encode().unwrap();
    let decoded = SendName::decode(&encoded[2..]).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn test_send_name_node_name_too_long() {
    let long_name = "a".repeat(300);
    let send_name = SendName::new(DistributionFlags::default_otp26(), 123, long_name);

    let result = send_name.encode();
    assert!(result.is_err());
}

//
// Status Handling
//

#[test]
fn test_status_is_ok() {
    assert!(Status::Ok.is_ok());
    assert!(Status::OkSimultaneous.is_ok());
    assert!(!Status::Nok.is_ok());
    assert!(!Status::NotAllowed.is_ok());
}

#[test]
fn test_status_from_u8() {
    assert_eq!(Status::from_u8(0), Some(Status::Ok));
    assert_eq!(Status::from_u8(1), Some(Status::OkSimultaneous));
    assert_eq!(Status::from_u8(2), Some(Status::Nok));
    assert_eq!(Status::from_u8(3), Some(Status::NotAllowed));
    assert_eq!(Status::from_u8(4), Some(Status::Alive));
    assert_eq!(Status::from_u8(99), None);
}

//
// ChallengeReply Verification
//

#[test]
fn test_challenge_reply_verify() {
    let reply = ChallengeReply::new(12345, 67890, "secret");
    assert!(reply.verify(67890, "secret"));
    assert!(!reply.verify(67890, "wrong"));
    assert!(!reply.verify(99999, "secret"));
}

#[test]
fn test_challenge_reply_roundtrip() {
    let reply = ChallengeReply::new(12345, 67890, "test_cookie");
    let encoded = reply.encode();
    let decoded = ChallengeReply::decode(&encoded[2..]).unwrap();

    assert_eq!(reply.challenge, decoded.challenge);
    assert_eq!(reply.digest, decoded.digest);
}

//
// ChallengeAck Verification
//

#[test]
fn test_challenge_ack_verify() {
    let ack = ChallengeAck::new(12345, "secret");
    assert!(ack.verify(12345, "secret"));
    assert!(!ack.verify(12345, "wrong"));
    assert!(!ack.verify(54321, "secret"));
}

#[test]
fn test_challenge_ack_roundtrip() {
    let ack = ChallengeAck::new(98765, "my_cookie");
    let encoded = ack.encode();
    let decoded = ChallengeAck::decode(&encoded[2..]).unwrap();

    assert_eq!(ack.digest, decoded.digest);
}
