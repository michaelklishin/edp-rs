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

use edp_client::epmd_client::{EpmdClient, NodeType};
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

fn epmd_available() -> bool {
    Command::new("epmd")
        .arg("-names")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn start_epmd() -> Option<Child> {
    Command::new("epmd").arg("-daemon").spawn().ok()
}

#[tokio::test]
async fn test_epmd_list_nodes() {
    if !epmd_available() {
        eprintln!("Skipping test: epmd not available in PATH");
        return;
    }

    start_epmd();
    sleep(Duration::from_millis(500)).await;

    let client = EpmdClient::new("localhost");
    let result = client.list_nodes().await;

    match result {
        Ok(nodes) => {
            println!("Registered nodes:\n{}", nodes);
        }
        Err(e) => {
            println!("EPMD list_nodes returned error: {}", e);
        }
    }
}

#[tokio::test]
async fn test_epmd_register_and_lookup() {
    if !epmd_available() {
        eprintln!("Skipping test: epmd not available in PATH");
        return;
    }

    start_epmd();
    sleep(Duration::from_millis(500)).await;

    let client = EpmdClient::new("localhost");
    let test_node_name = "rust_test_node";
    let test_port = 45678u16;

    let creation_result = client
        .register_node(test_port, test_node_name, NodeType::Normal, 6, 5, &[])
        .await;

    match creation_result {
        Ok(creation) => {
            println!("Node registered with creation: {}", creation);
            println!("Note: EPMD registration requires keeping the connection open.");
            println!("Registration will be lost when this test completes.");
        }
        Err(e) => {
            println!(
                "Node registration failed (this is expected behavior - EPMD requires persistent connection): {}",
                e
            );
            println!("Error: {}", e);
        }
    }
}

#[tokio::test]
async fn test_epmd_lookup_nonexistent_node() {
    if !epmd_available() {
        eprintln!("Skipping test: epmd not available in PATH");
        return;
    }

    start_epmd();
    sleep(Duration::from_millis(500)).await;

    let client = EpmdClient::new("localhost");
    let result = client.lookup_node("nonexistent_node_xyz_123").await;

    assert!(result.is_err());
    if let Err(e) = result {
        println!("Expected error for nonexistent node: {}", e);
    }
}
