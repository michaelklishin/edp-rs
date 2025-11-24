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

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!(
            "Usage: send_message_to_registered_process <peer_node> <destination_name> <message>"
        );
        eprintln!("Example: send_message_to_registered_process foo@localhost shell hello");
        std::process::exit(1);
    }

    let peer_node = &args[1];
    let destination = &args[2];
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

    let message = OwnedTerm::Atom(Atom::new(message_text));
    let destination_atom = Atom::new(destination);

    node.send_to_name(&destination_atom, message)
        .await
        .context("Failed to send message")?;

    println!(
        "Sent message '{}' to registered process '{}'",
        message_text, destination
    );

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(())
}
