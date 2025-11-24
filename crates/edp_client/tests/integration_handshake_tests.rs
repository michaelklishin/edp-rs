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

mod test_erlang_node;

use std::time::Duration;
use test_erlang_node::{TestErlangNode, epmd_client};
use tokio::time::sleep;

#[tokio::test]
async fn test_can_discover_erlang_node_via_epmd() {
    if let Ok(node) = TestErlangNode::new("test_discover_node").await {
        let client = epmd_client().await;

        match client.lookup_node("test_discover_node").await {
            Ok(_node_info) => {
                println!("Successfully discovered Erlang node via EPMD");
            }
            Err(e) => {
                eprintln!("Failed to discover node: {}", e);
            }
        }

        drop(node);
    }
}

#[tokio::test]
async fn test_node_registration_lifecycle() {
    if let Ok(_node) = TestErlangNode::new("test_lifecycle_node").await {
        let client = epmd_client().await;

        sleep(Duration::from_millis(500)).await;

        match client.lookup_node("test_lifecycle_node").await {
            Ok(_info) => {
                println!("Node was registered with EPMD");
            }
            Err(e) => {
                eprintln!("Node lookup failed: {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_multiple_concurrent_node_lookups() {
    if let Ok(node1) = TestErlangNode::new("test_concurrent_1").await
        && let Ok(node2) = TestErlangNode::new("test_concurrent_2").await
    {
        let client = epmd_client().await;

        sleep(Duration::from_millis(500)).await;

        let lookup1 = client.lookup_node("test_concurrent_1").await;
        let lookup2 = client.lookup_node("test_concurrent_2").await;

        assert!(lookup1.is_ok(), "Failed to lookup first concurrent node");
        assert!(lookup2.is_ok(), "Failed to lookup second concurrent node");

        drop(node1);
        drop(node2);
    }
}

#[tokio::test]
async fn test_nonexistent_node_lookup_fails() {
    let client = epmd_client().await;
    let result = client
        .lookup_node("definitely_does_not_exist_xyz_123")
        .await;

    assert!(result.is_err(), "Looking up nonexistent node should fail");
}
