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
use erltf::types::Atom;
use erltf::OwnedTerm;
use std::env;
use std::process;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: send_direct_message <peer_node> <target_process> <message>");
        eprintln!("Example: send_direct_message foo@localhost shell hello_world");
        eprintln!("Example: send_direct_message rabbit@localhost rabbit hello_from_rust");
        eprintln!();
        eprintln!(
            "This example demonstrates sending a message to a registered process on a remote node"
        );
        eprintln!("using the RegSend control message (via node.send_to_name()).");
        eprintln!();
        eprintln!("Start an Erlang node first:");
        eprintln!("  erl -name foo@localhost -setcookie monster");
        eprintln!();
        eprintln!("Common registered processes:");
        eprintln!("  - Erlang shell: 'shell' (only on interactive erl nodes)");
        eprintln!("  - RabbitMQ: 'rabbit', 'rabbit_node_monitor', etc.");
        process::exit(1);
    }

    let peer_node = &args[1];
    let target_name = &args[2];
    let message_text = &args[3];

    let cookie = env::var("ERLANG_COOKIE").unwrap_or_else(|_| "monster".to_string());

    let local_node_name = format!(
        "rust_sender@{}",
        peer_node.split('@').nth(1).unwrap_or("localhost")
    );

    let mut node = Node::new(local_node_name, cookie);
    node.start(0).await.context("Failed to start local node")?;

    println!("Connecting to {}...", peer_node);
    node.connect(peer_node)
        .await
        .context("Failed to connect to peer node")?;
    println!("Connected to {}", peer_node);

    let target_process = Atom::new(target_name);
    let message = OwnedTerm::tuple(vec![
        OwnedTerm::Atom(Atom::new("rust_message")),
        OwnedTerm::Atom(Atom::new(message_text)),
    ]);

    println!(
        "Sending message to registered process '{}' on {}...",
        target_process.as_str(),
        peer_node
    );

    let result = node
        .rpc_call(
            peer_node,
            "erlang",
            "send",
            vec![OwnedTerm::Atom(target_process.clone()), message.clone()],
        )
        .await;

    match result {
        Ok(response) => {
            println!(
                "Sent message '{}' to registered process '{}' - RPC response: {:?}",
                message_text,
                target_process.as_str(),
                response
            );
        }
        Err(e) => {
            eprintln!("Failed to send message: {}", e);
            eprintln!(
                "Note: The process '{}' may not be registered on the remote node",
                target_name
            );
        }
    }

    sleep(Duration::from_millis(100)).await;

    Ok(())
}
