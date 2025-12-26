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

//! Elixir echo example demonstrating Option<T> interop with nil.
//!
//! # Usage
//!
//! 1. Start the Elixir echo server:
//!    ```sh
//!    cd crates/edp_examples_elixir/elixir
//!    elixir --sname echo --cookie secret echo_server.exs
//!    ```
//!
//! 2. Run this example:
//!    ```sh
//!    cargo run --package edp_examples_elixir --bin example_elixir_echo
//!    ```

use anyhow::{Context, Result};
use edp_node::Node;
use erltf::OwnedTerm;
use erltf::types::Atom;
use erltf_serde::{from_term, to_term};
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> Result<()> {
    let cookie = "secret";
    let elixir_node = "echo@localhost";
    let client_node = "rust_echo_client@localhost";

    println!("Connecting to {}", elixir_node);

    let mut node = Node::new(client_node.to_string(), cookie.to_string());
    node.start(0).await.context("Failed to start node")?;
    node.connect(elixir_node)
        .await
        .context("Failed to connect")?;

    println!("Connected!\n");

    // Basic types
    echo(
        &mut node,
        elixir_node,
        "atom :hello",
        OwnedTerm::Atom(Atom::new("hello")),
    )
    .await?;
    echo(&mut node, elixir_node, "integer 42", OwnedTerm::Integer(42)).await?;
    echo(
        &mut node,
        elixir_node,
        "binary",
        OwnedTerm::Binary(b"Hello from Rust!".to_vec()),
    )
    .await?;

    // Compound types
    echo(
        &mut node,
        elixir_node,
        "tuple {:ok, 123}",
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("ok")),
            OwnedTerm::Integer(123),
        ]),
    )
    .await?;

    echo(
        &mut node,
        elixir_node,
        "list [1, 2, 3]",
        OwnedTerm::List(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::Integer(2),
            OwnedTerm::Integer(3),
        ]),
    )
    .await?;

    let mut map = BTreeMap::new();
    map.insert(
        OwnedTerm::Atom(Atom::new("key")),
        OwnedTerm::Binary(b"value".to_vec()),
    );
    echo(
        &mut node,
        elixir_node,
        "map %{key: \"value\"}",
        OwnedTerm::Map(map),
    )
    .await?;

    // Option<T> with elixir-interop: None becomes nil
    println!("\n-- Option<T> with elixir-interop --");

    let none: Option<i32> = None;
    let term = to_term(&none)?;
    println!("Sending None (serialized as {:?})", term);
    let response = node
        .rpc_call(elixir_node, "Elixir.EchoServer", "echo", vec![term])
        .await?;
    let response_val: Option<i32> = from_term(&response)?;
    println!("Received: {:?}\n", response_val);

    let some: Option<i32> = Some(42);
    let term = to_term(&some)?;
    println!("Sending Some(42) (serialized as {:?})", term);
    let response = node
        .rpc_call(elixir_node, "Elixir.EchoServer", "echo", vec![term])
        .await?;
    let response_val: Option<i32> = from_term(&response)?;
    println!("Received: {:?}\n", response_val);

    println!("Done!");
    Ok(())
}

async fn echo(node: &mut Node, target: &str, desc: &str, term: OwnedTerm) -> Result<()> {
    println!("Sending {}", desc);
    let response = node
        .rpc_call(target, "Elixir.EchoServer", "echo", vec![term])
        .await?;
    println!("Received: {:?}\n", response);
    Ok(())
}
