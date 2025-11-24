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

use edp_client::{Connection, ConnectionConfig, ConnectionState, Creation};

#[test]
fn test_connection_initial_state() {
    let config = ConnectionConfig::new("node1@localhost", "node2@localhost", "secret");
    let conn = Connection::new(config);

    assert_eq!(conn.state(), ConnectionState::Disconnected);
    assert!(!conn.is_connected());
    assert!(conn.negotiated_flags().is_none());
}

#[test]
fn test_connection_config_builder() {
    let config = ConnectionConfig::new("node1@localhost", "node2@localhost", "secret")
        .with_epmd_host("127.0.0.1")
        .with_creation(42)
        .with_timeout(std::time::Duration::from_secs(5));

    assert_eq!(config.local_node_name, "node1@localhost");
    assert_eq!(config.remote_node_name, "node2@localhost");
    assert_eq!(config.cookie, "secret");
    assert_eq!(config.epmd_host, "127.0.0.1");
    assert_eq!(config.creation, Creation::new(42));
    assert_eq!(config.timeout, std::time::Duration::from_secs(5));
}

#[test]
fn test_connection_state_as_str() {
    assert_eq!(ConnectionState::Disconnected.as_str(), "disconnected");
    assert_eq!(ConnectionState::Connecting.as_str(), "connecting");
    assert_eq!(ConnectionState::SendingName.as_str(), "sending_name");
    assert_eq!(ConnectionState::AwaitingStatus.as_str(), "awaiting_status");
    assert_eq!(
        ConnectionState::AwaitingChallenge.as_str(),
        "awaiting_challenge"
    );
    assert_eq!(
        ConnectionState::SendingChallengeReply.as_str(),
        "sending_challenge_reply"
    );
    assert_eq!(
        ConnectionState::AwaitingChallengeAck.as_str(),
        "awaiting_challenge_ack"
    );
    assert_eq!(ConnectionState::Connected.as_str(), "connected");
    assert_eq!(ConnectionState::Failed.as_str(), "failed");
}
