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

use edp_client::errors::Error;
use edp_client::state_machine::ConnectionState;

#[test]
fn test_invalid_state_transition_uses_enum() {
    let error = Error::InvalidStateTransition {
        from: ConnectionState::Disconnected,
        to: ConnectionState::Connected,
    };

    let error_msg = error.to_string();
    assert!(error_msg.contains("disconnected"));
    assert!(error_msg.contains("connected"));
    assert!(error_msg.contains("->"));
}

#[test]
fn test_invalid_state_uses_enum() {
    let error = Error::InvalidState {
        state: ConnectionState::SendingName,
    };

    let error_msg = error.to_string();
    assert!(error_msg.contains("sending_name"));
}

#[test]
fn test_invalid_state_message_for_custom_errors() {
    let error = Error::InvalidStateMessage("no active stream".to_string());

    let error_msg = error.to_string();
    assert_eq!(error_msg, "no active stream");
}

#[test]
fn test_connection_state_display() {
    assert_eq!(ConnectionState::Disconnected.to_string(), "disconnected");
    assert_eq!(ConnectionState::Connecting.to_string(), "connecting");
    assert_eq!(ConnectionState::SendingName.to_string(), "sending_name");
    assert_eq!(
        ConnectionState::AwaitingStatus.to_string(),
        "awaiting_status"
    );
    assert_eq!(
        ConnectionState::AwaitingChallenge.to_string(),
        "awaiting_challenge"
    );
    assert_eq!(
        ConnectionState::SendingChallengeReply.to_string(),
        "sending_challenge_reply"
    );
    assert_eq!(
        ConnectionState::AwaitingChallengeAck.to_string(),
        "awaiting_challenge_ack"
    );
    assert_eq!(ConnectionState::Connected.to_string(), "connected");
    assert_eq!(ConnectionState::Failed.to_string(), "failed");
}

#[test]
fn test_connection_state_as_str() {
    assert_eq!(ConnectionState::Disconnected.as_str(), "disconnected");
    assert_eq!(ConnectionState::Connected.as_str(), "connected");
}

#[test]
fn test_all_connection_states_are_copy() {
    let state1 = ConnectionState::Connected;
    let state2 = state1;
    assert_eq!(state1, state2);
}

#[test]
fn test_connection_state_equality() {
    assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
    assert_ne!(ConnectionState::Disconnected, ConnectionState::Connected);
}

#[test]
fn test_error_debug_with_enum_state() {
    let error = Error::InvalidState {
        state: ConnectionState::Failed,
    };

    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("InvalidState"));
    assert!(debug_str.contains("Failed"));
}
