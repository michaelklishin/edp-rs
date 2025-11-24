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

use edp_node::Node;

//
// Reference Creation Tests
//

#[tokio::test]
async fn test_rpc_reference_creation() {
    let mut node = Node::new("rpc_test1@localhost", "secret");
    node.start(5580).await.unwrap();

    let ref1 = node.make_reference();
    let ref2 = node.make_reference();

    assert_ne!(ref1, ref2);
    assert_eq!(ref1.node.as_str(), "rpc_test1@localhost");
    assert_eq!(ref2.node.as_str(), "rpc_test1@localhost");
}

#[tokio::test]
async fn test_rpc_reference_uniqueness() {
    let mut node = Node::new("rpc_test1b@localhost", "secret");
    node.start(5580).await.unwrap();

    let mut refs = Vec::new();
    for _ in 0..100 {
        refs.push(node.make_reference());
    }

    for i in 0..refs.len() {
        for j in (i + 1)..refs.len() {
            assert_ne!(
                refs[i], refs[j],
                "References at positions {} and {} should be unique",
                i, j
            );
        }
    }
}

//
// RPC Call Tests
//

#[tokio::test]
async fn test_rpc_to_nonexistent_node() {
    let mut node = Node::new("rpc_test2@localhost", "secret");
    node.start(5581).await.unwrap();

    let result = node
        .rpc_call("nonexistent@localhost", "erlang", "node", vec![])
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_rpc_requires_connection() {
    let mut node = Node::new("rpc_test3@localhost", "secret");
    node.start(5582).await.unwrap();

    let result = node
        .rpc_call("not_connected@localhost", "erlang", "node", vec![])
        .await;

    assert!(result.is_err());
}

//
// Node Metadata Tests
//

#[tokio::test]
async fn test_rpc_node_name_and_cookie_accessors() {
    let node = Node::new("rpc_test4@localhost", "secret_cookie");

    assert_eq!(node.name().as_str(), "rpc_test4@localhost");
    assert_eq!(node.cookie(), "secret_cookie");
}

#[tokio::test]
async fn test_rpc_creation_increments() {
    let mut node = Node::new("rpc_test5@localhost", "secret");
    node.start(5583).await.unwrap();

    let initial_creation = node.creation();
    assert!(initial_creation > 0);
}
