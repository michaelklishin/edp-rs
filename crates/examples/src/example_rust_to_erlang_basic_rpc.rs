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

use anyhow::{Context, Result};
use edp_node::Node;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .init();

    let cookie = "monster";
    let node_name = "test@sunnyside";
    let client_node_name = "rust_rpc_client@sunnyside";

    println!("Basic Rust-to-Erlang RPC Example");
    println!("Connecting to Erlang node: {}", node_name);

    debug!("Starting client node");
    let mut node = Node::new(client_node_name.to_string(), cookie.to_string());
    node.start(0).await.context("Failed to start node")?;

    debug!("Connecting to remote node");
    node.connect(node_name)
        .await
        .context("Failed to connect to test node")?;

    info!("Connected successfully!");
    println!("Connected successfully!");

    // Simple test_function/0 test
    println!("\nCalling test_node:test_function/0...");
    debug!("Making RPC call: test_node:test_function()");
    let response = node
        .rpc_call(node_name, "test_node", "test_function", vec![])
        .await
        .context("Failed to call test_node:test_function/0")?;

    println!("Response: {:#?}", response);
    println!("\nTest completed successfully!");

    Ok(())
}
