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
use erltf::types::Atom;
use std::sync::Arc;
use tokio::sync::Mutex;

struct CollectorProcess {
    messages: Arc<Mutex<Vec<OwnedTerm>>>,
}

impl CollectorProcess {
    fn new(messages: Arc<Mutex<Vec<OwnedTerm>>>) -> Self {
        Self { messages }
    }
}

impl Process for CollectorProcess {
    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        if let Message::Regular { body, .. } = msg {
            self.messages.lock().await.push(body);
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_local_message_passing() {
    let mut node = Node::new("test@localhost", "secret");
    node.start(5555).await.unwrap();

    let messages = Arc::new(Mutex::new(Vec::new()));
    let collector = CollectorProcess::new(messages.clone());

    let pid = node.spawn(collector).await.unwrap();

    node.send(&pid, OwnedTerm::Atom(Atom::new("hello")))
        .await
        .unwrap();
    node.send(&pid, OwnedTerm::Integer(42)).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let collected = messages.lock().await;
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0], OwnedTerm::Atom(Atom::new("hello")));
    assert_eq!(collected[1], OwnedTerm::Integer(42));
}

#[tokio::test]
async fn test_registered_name_message_passing() {
    let mut node = Node::new("test2@localhost", "secret");
    node.start(5556).await.unwrap();

    let messages = Arc::new(Mutex::new(Vec::new()));
    let collector = CollectorProcess::new(messages.clone());

    let pid = node.spawn(collector).await.unwrap();
    node.register(Atom::new("collector"), pid).await.unwrap();

    node.send_to_name(&Atom::new("collector"), OwnedTerm::Atom(Atom::new("test")))
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let collected = messages.lock().await;
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0], OwnedTerm::Atom(Atom::new("test")));
}

#[tokio::test]
async fn test_message_to_nonexistent_process() {
    let mut node = Node::new("test3@localhost", "secret");
    node.start(5557).await.unwrap();

    let result = node
        .send(
            &erltf::types::ExternalPid::new(Atom::new("test3@localhost"), 99999, 0, 1),
            OwnedTerm::Atom(Atom::new("test")),
        )
        .await;

    assert!(result.is_err());
}
