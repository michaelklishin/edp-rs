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
use edp_node::{Message, Node, Process};
use erltf::types::{Atom, ExternalPid};
use erltf::OwnedTerm;
use std::env;
use std::process;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
struct LinkExampleProcess {
    exit_received: Arc<Mutex<Vec<(ExternalPid, OwnedTerm)>>>,
}

impl LinkExampleProcess {
    fn new(exit_received: Arc<Mutex<Vec<(ExternalPid, OwnedTerm)>>>) -> Self {
        Self { exit_received }
    }
}

impl Process for LinkExampleProcess {
    async fn handle_message(&mut self, msg: Message) -> edp_node::Result<()> {
        match msg {
            Message::Exit { from, reason } => {
                println!("Exit signal received from {}: reason={:?}", from, reason);
                self.exit_received.lock().await.push((from, reason));
                Ok(())
            }
            Message::Regular { from, body } => {
                println!("Regular message from {:?}: {:?}", from, body);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: link_processes <peer_node> <target_process>");
        eprintln!("Example: link_processes foo@localhost shell");
        eprintln!("Example: link_processes rabbit@localhost rabbit");
        eprintln!();
        eprintln!("This example demonstrates linking and unlinking processes using Link");
        eprintln!("and UnlinkId/UnlinkIdAck control messages (via node.link() and node.unlink()).");
        eprintln!();
        eprintln!("Start an Erlang node first:");
        eprintln!("  erl -name foo@localhost -setcookie monster");
        process::exit(1);
    }

    let peer_node = &args[1];
    let target_name = &args[2];

    let cookie = env::var("ERLANG_COOKIE").unwrap_or_else(|_| "monster".to_string());

    let local_node_name = format!(
        "rust_linker@{}",
        peer_node.split('@').nth(1).unwrap_or("localhost")
    );

    let mut node = Node::new(local_node_name, cookie);
    node.start(0).await.context("Failed to start local node")?;

    println!("Connecting to {}...", peer_node);
    node.connect(peer_node)
        .await
        .context("Failed to connect to peer node")?;
    println!("Connected to {}", peer_node);

    let exit_received = Arc::new(Mutex::new(Vec::new()));
    let process = LinkExampleProcess::new(exit_received.clone());

    let local_pid = node
        .spawn(process)
        .await
        .context("Failed to spawn local process")?;

    node.register(Atom::new("link_example"), local_pid.clone())
        .await
        .context("Failed to register local process")?;

    println!("Local process registered as 'link_example'");

    let peer_pid_result = node
        .rpc_call(
            peer_node,
            "erlang",
            "whereis",
            vec![OwnedTerm::Atom(Atom::new(target_name))],
        )
        .await
        .context("Failed to call whereis on remote node")?;

    let peer_shell_pid = match peer_pid_result {
        OwnedTerm::Pid(pid) => pid,
        OwnedTerm::Atom(ref atom) if atom.as_str() == "undefined" => {
            anyhow::bail!(
                "Process '{}' is not registered on the remote node '{}'",
                target_name,
                peer_node
            )
        }
        OwnedTerm::Tuple(ref elements) if elements.len() == 2 => match &elements[1] {
            OwnedTerm::Pid(pid) => pid.clone(),
            OwnedTerm::Atom(ref atom) if atom.as_str() == "undefined" => {
                anyhow::bail!(
                    "Process '{}' is not registered on the remote node '{}'",
                    target_name,
                    peer_node
                )
            }
            _ => anyhow::bail!("Unexpected RPC response: {:?}", peer_pid_result),
        },
        _ => anyhow::bail!("Unexpected response from whereis: {:?}", peer_pid_result),
    };

    println!("Found remote process '{}': {}", target_name, peer_shell_pid);

    println!("Linking to remote shell process using Link control message...");
    node.link(&local_pid, &peer_shell_pid)
        .await
        .context("Failed to link to remote process")?;

    println!("Linked successfully!");
    println!("Waiting for 5 seconds...");

    sleep(Duration::from_secs(5)).await;

    println!("Unlinking from remote shell process using UnlinkId control message...");
    node.unlink(&local_pid, &peer_shell_pid)
        .await
        .context("Failed to unlink from remote process")?;

    println!("Unlinked successfully!");

    println!("Sending a test message to verify connection...");
    node.send(
        &peer_shell_pid,
        OwnedTerm::Atom(Atom::new("link_test_complete")),
    )
    .await
    .context("Failed to send test message")?;

    sleep(Duration::from_millis(100)).await;

    Ok(())
}
