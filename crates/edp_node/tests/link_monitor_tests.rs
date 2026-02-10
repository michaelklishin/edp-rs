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

use edp_node::{Message, Node, Process, Result};
use erltf::OwnedTerm;
use erltf::types::{ExternalPid, ExternalReference};
use std::sync::Arc;
use tokio::sync::Mutex;

fn test_node_name(base: &str) -> String {
    format!("{}_{}@localhost", base, std::process::id())
}

#[derive(Clone)]
struct LinkTestProcess {
    exit_received: Arc<Mutex<Vec<(ExternalPid, OwnedTerm)>>>,
    monitor_exit_received: Arc<Mutex<Vec<(ExternalPid, ExternalReference, OwnedTerm)>>>,
}

impl LinkTestProcess {
    fn new(
        exit_received: Arc<Mutex<Vec<(ExternalPid, OwnedTerm)>>>,
        monitor_exit_received: Arc<Mutex<Vec<(ExternalPid, ExternalReference, OwnedTerm)>>>,
    ) -> Self {
        Self {
            exit_received,
            monitor_exit_received,
        }
    }
}

impl Process for LinkTestProcess {
    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Exit { from, reason } => {
                self.exit_received.lock().await.push((from, reason));
                Ok(())
            }
            Message::MonitorExit {
                monitored,
                reference,
                reason,
            } => {
                self.monitor_exit_received
                    .lock()
                    .await
                    .push((monitored, reference, reason));
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

//
// Link Tests
//

#[tokio::test]
async fn test_link_tracking() {
    let mut node = Node::new(test_node_name("test_link"), "secret");
    node.start(0).await.unwrap();

    let exits = Arc::new(Mutex::new(Vec::new()));
    let monitor_exits = Arc::new(Mutex::new(Vec::new()));

    let process1 = LinkTestProcess::new(exits.clone(), monitor_exits.clone());
    let process2 = LinkTestProcess::new(exits.clone(), monitor_exits.clone());

    let pid1 = node.spawn(process1).await.unwrap();
    let pid2 = node.spawn(process2).await.unwrap();

    node.link(&pid1, &pid2).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_unlink() {
    let mut node = Node::new(test_node_name("test_unlink"), "secret");
    node.start(0).await.unwrap();

    let exits = Arc::new(Mutex::new(Vec::new()));
    let monitor_exits = Arc::new(Mutex::new(Vec::new()));

    let process1 = LinkTestProcess::new(exits.clone(), monitor_exits.clone());
    let process2 = LinkTestProcess::new(exits.clone(), monitor_exits.clone());

    let pid1 = node.spawn(process1).await.unwrap();
    let pid2 = node.spawn(process2).await.unwrap();

    node.link(&pid1, &pid2).await.unwrap();
    node.unlink(&pid1, &pid2).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

//
// Monitor Tests
//

#[tokio::test]
async fn test_monitor_tracking() {
    let mut node = Node::new(test_node_name("test_monitor"), "secret");
    node.start(0).await.unwrap();

    let exits = Arc::new(Mutex::new(Vec::new()));
    let monitor_exits = Arc::new(Mutex::new(Vec::new()));

    let process1 = LinkTestProcess::new(exits.clone(), monitor_exits.clone());
    let process2 = LinkTestProcess::new(exits.clone(), monitor_exits.clone());

    let pid1 = node.spawn(process1).await.unwrap();
    let pid2 = node.spawn(process2).await.unwrap();

    let reference = node.monitor(&pid1, &pid2).await.unwrap();

    assert_eq!(reference.ids.len(), 3);

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_demonitor() {
    let mut node = Node::new(test_node_name("test_demonitor"), "secret");
    node.start(0).await.unwrap();

    let exits = Arc::new(Mutex::new(Vec::new()));
    let monitor_exits = Arc::new(Mutex::new(Vec::new()));

    let process1 = LinkTestProcess::new(exits.clone(), monitor_exits.clone());
    let process2 = LinkTestProcess::new(exits.clone(), monitor_exits.clone());

    let pid1 = node.spawn(process1).await.unwrap();
    let pid2 = node.spawn(process2).await.unwrap();

    let reference = node.monitor(&pid1, &pid2).await.unwrap();
    node.demonitor(&pid1, &pid2, &reference).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}
