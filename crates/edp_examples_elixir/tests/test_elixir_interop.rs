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
use edp_node::Node;
use erltf::OwnedTerm;
use erltf::types::Atom;
use erltf_serde::{from_term, to_term};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

const COOKIE: &str = "test_cookie";

struct ElixirNode {
    child: Child,
    name: String,
}

impl ElixirNode {
    fn start(short_name: &str) -> Result<Self> {
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("elixir")
            .join("echo_server.exs");

        let child = Command::new("elixir")
            .args(["--sname", short_name, "--cookie", COOKIE])
            .arg(&script)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        thread::sleep(Duration::from_millis(1000));

        let hostname = hostname::get()?.to_string_lossy().to_string();
        let name = format!("{}@{}", short_name, hostname);

        Ok(Self { child, name })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for ElixirNode {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

async fn setup_client(test_name: &str, elixir_node: &str) -> Result<Node> {
    let hostname = hostname::get()?.to_string_lossy().to_string();
    let client_name = format!("rust_test_{}@{}", test_name, hostname);

    let mut node = Node::new(client_name, COOKIE.to_string());
    node.start(0).await?;
    node.connect(elixir_node).await?;
    Ok(node)
}

#[tokio::test]
async fn test_elixir_echo_nil() -> Result<()> {
    let elixir = ElixirNode::start("elixir_test_nil")?;
    let node = setup_client("nil", elixir.name()).await?;

    let none: Option<i32> = None;
    let term = to_term(&none)?;

    // Verify it serializes as nil atom
    assert!(matches!(&term, OwnedTerm::Atom(a) if a.as_str() == "nil"));

    let response = node
        .rpc_call(elixir.name(), "Elixir.TestEchoServer", "echo", vec![term])
        .await?;

    let result: Option<i32> = from_term(&response)?;
    assert_eq!(result, None);

    Ok(())
}

#[tokio::test]
async fn test_elixir_echo_some() -> Result<()> {
    let elixir = ElixirNode::start("elixir_test_some")?;
    let node = setup_client("some", elixir.name()).await?;

    let some: Option<i32> = Some(42);
    let term = to_term(&some)?;

    let response = node
        .rpc_call(elixir.name(), "Elixir.TestEchoServer", "echo", vec![term])
        .await?;

    let result: Option<i32> = from_term(&response)?;
    assert_eq!(result, Some(42));

    Ok(())
}

#[tokio::test]
async fn test_elixir_echo_atom() -> Result<()> {
    let elixir = ElixirNode::start("elixir_test_atom")?;
    let node = setup_client("atom", elixir.name()).await?;

    let term = OwnedTerm::Atom(Atom::new("hello"));
    let response = node
        .rpc_call(elixir.name(), "Elixir.TestEchoServer", "echo", vec![term])
        .await?;

    assert!(matches!(response, OwnedTerm::Atom(a) if a.as_str() == "hello"));

    Ok(())
}

#[tokio::test]
async fn test_elixir_echo_option_string() -> Result<()> {
    let elixir = ElixirNode::start("elixir_test_opt_str")?;
    let node = setup_client("opt_str", elixir.name()).await?;

    let none: Option<String> = None;
    let term = to_term(&none)?;
    let response = node
        .rpc_call(elixir.name(), "Elixir.TestEchoServer", "echo", vec![term])
        .await?;
    let result: Option<String> = from_term(&response)?;
    assert_eq!(result, None);

    let some: Option<String> = Some("hello".to_string());
    let term = to_term(&some)?;
    let response = node
        .rpc_call(elixir.name(), "Elixir.TestEchoServer", "echo", vec![term])
        .await?;
    let result: Option<String> = from_term(&response)?;
    assert_eq!(result, Some("hello".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_elixir_echo_tagged() -> Result<()> {
    let elixir = ElixirNode::start("elixir_test_tagged")?;
    let node = setup_client("tagged", elixir.name()).await?;

    let term = OwnedTerm::Atom(Atom::new("world"));
    let response = node
        .rpc_call(
            elixir.name(),
            "Elixir.TestEchoServer",
            "echo_tagged",
            vec![term],
        )
        .await?;

    match response {
        OwnedTerm::Tuple(elems) if elems.len() == 2 => {
            assert!(matches!(&elems[0], OwnedTerm::Atom(a) if a.as_str() == "echo"));
            assert!(matches!(&elems[1], OwnedTerm::Atom(a) if a.as_str() == "world"));
        }
        _ => panic!("Expected a 2-tuple"),
    }

    Ok(())
}
