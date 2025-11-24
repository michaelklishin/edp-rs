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

use anyhow::Result;
use edp_node::{GenServer, GenServerProcess, Message, Node, Process};
use erltf::types::Atom;
use erltf::OwnedTerm;

struct CounterServer {
    count: i64,
}

impl CounterServer {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl GenServer for CounterServer {
    async fn init(&mut self, _args: Vec<OwnedTerm>) -> edp_node::Result<()> {
        tracing::info!("Counter initialized");
        Ok(())
    }

    async fn handle_call(
        &mut self,
        msg: OwnedTerm,
        _from: erltf::types::ExternalPid,
    ) -> edp_node::Result<edp_node::CallResult> {
        match msg {
            OwnedTerm::Atom(ref atom) if atom.as_str() == "get" => {
                Ok(edp_node::CallResult::Reply(OwnedTerm::Integer(self.count)))
            }
            _ => Ok(edp_node::CallResult::NoReply),
        }
    }

    async fn handle_cast(&mut self, msg: OwnedTerm) -> edp_node::Result<()> {
        match msg {
            OwnedTerm::Atom(ref atom) if atom.as_str() == "increment" => {
                self.count += 1;
                tracing::info!("Counter incremented to {}", self.count);
            }
            OwnedTerm::Atom(ref atom) if atom.as_str() == "decrement" => {
                self.count -= 1;
                tracing::info!("Counter decremented to {}", self.count);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_info(&mut self, msg: OwnedTerm) -> edp_node::Result<()> {
        tracing::info!("Received info message: {:?}", msg);
        Ok(())
    }
}

struct EchoProcess;

impl Process for EchoProcess {
    async fn handle_message(&mut self, msg: Message) -> edp_node::Result<()> {
        if let Message::Regular { body, .. } = msg {
            tracing::info!("Echo received: {:?}", body);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let mut node = Node::new("test@localhost", "secret");

    tracing::info!("Starting node...");
    node.start(25672).await?;

    tracing::info!("Spawning echo process...");
    let echo_pid = node.spawn(EchoProcess).await?;
    tracing::info!("Echo process PID: {:?}", echo_pid);

    node.register(Atom::new("echo"), echo_pid.clone()).await?;
    tracing::info!("Registered echo process as 'echo'");

    tracing::info!("Spawning counter server...");
    let counter = CounterServer::new();
    let counter_process = GenServerProcess::new(counter, node.registry());
    let counter_pid = node.spawn(counter_process).await?;
    tracing::info!("Counter server PID: {:?}", counter_pid);

    node.register(Atom::new("counter"), counter_pid.clone())
        .await?;
    tracing::info!("Registered counter server as 'counter'");

    tracing::info!("Sending test message to echo...");
    node.send(&echo_pid, OwnedTerm::Atom(Atom::new("hello")))
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    tracing::info!("Node running with {} processes", node.process_count().await);
    tracing::info!("Registered names: {:?}", node.registered().await);

    tracing::info!("Press Ctrl+C to exit");
    tokio::signal::ctrl_c().await?;

    Ok(())
}
