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

use edp_node::{Message, Node, Process};
use erltf::OwnedTerm;
use erltf::types::Atom;

struct TestProcess {
    received: Vec<OwnedTerm>,
}

impl TestProcess {
    fn new() -> Self {
        Self {
            received: Vec::new(),
        }
    }
}

impl Process for TestProcess {
    async fn handle_message(&mut self, msg: Message) -> edp_node::Result<()> {
        if let Message::Regular { body, .. } = msg {
            self.received.push(body);
        }
        Ok(())
    }
}

//
// Process Registration Tests
//

#[tokio::test]
async fn test_spawn_and_register_process() {
    let mut node = Node::new("test@localhost", "secret");
    node.start(5570).await.unwrap();

    let process = TestProcess::new();
    let pid = node.spawn(process).await.unwrap();

    assert!(
        node.register(Atom::new("test_proc"), pid.clone())
            .await
            .is_ok()
    );
    assert_eq!(node.whereis(&Atom::new("test_proc")).await, Some(pid));
}

#[tokio::test]
async fn test_unregister_process() {
    let mut node = Node::new("test2@localhost", "secret");
    node.start(5571).await.unwrap();

    let process = TestProcess::new();
    let pid = node.spawn(process).await.unwrap();

    node.register(Atom::new("test_proc"), pid.clone())
        .await
        .unwrap();
    node.unregister(&Atom::new("test_proc")).await.unwrap();

    assert_eq!(node.whereis(&Atom::new("test_proc")).await, None);
}

#[tokio::test]
async fn test_duplicate_registration_fails() {
    let mut node = Node::new("test3@localhost", "secret");
    node.start(5572).await.unwrap();

    let process1 = TestProcess::new();
    let pid1 = node.spawn(process1).await.unwrap();

    let process2 = TestProcess::new();
    let pid2 = node.spawn(process2).await.unwrap();

    node.register(Atom::new("test_proc"), pid1).await.unwrap();
    assert!(node.register(Atom::new("test_proc"), pid2).await.is_err());
}

#[tokio::test]
async fn test_list_registered_names() {
    let mut node = Node::new("test4@localhost", "secret");
    node.start(5573).await.unwrap();

    let process1 = TestProcess::new();
    let pid1 = node.spawn(process1).await.unwrap();
    node.register(Atom::new("proc1"), pid1).await.unwrap();

    let process2 = TestProcess::new();
    let pid2 = node.spawn(process2).await.unwrap();
    node.register(Atom::new("proc2"), pid2).await.unwrap();

    let registered = node.registered().await;
    assert_eq!(registered.len(), 2);
    assert!(registered.contains(&Atom::new("proc1")));
    assert!(registered.contains(&Atom::new("proc2")));
}

//
// Process Management Tests
//

#[tokio::test]
async fn test_process_count() {
    let mut node = Node::new("test5@localhost", "secret");
    node.start(5574).await.unwrap();

    assert_eq!(node.process_count().await, 0);

    let _pid1 = node.spawn(TestProcess::new()).await.unwrap();
    assert_eq!(node.process_count().await, 1);

    let _pid2 = node.spawn(TestProcess::new()).await.unwrap();
    assert_eq!(node.process_count().await, 2);
}
