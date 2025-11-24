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
use edp_client::epmd_client::EpmdClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: epmd_node_info <node_name>");
        eprintln!("Example: epmd_node_info rabbit");
        std::process::exit(1);
    }

    let node_name = &args[1];
    let client = EpmdClient::new("localhost");

    match client.lookup_node(node_name).await {
        Ok(node_info) => {
            println!("Node: {}", node_info.node_name);
            println!("Port: {}", node_info.port);
            println!("Node type: {:?}", node_info.node_type);
            println!("Protocol: {:?}", node_info.protocol);
            println!("Highest version: {}", node_info.highest_version);
            println!("Lowest version: {}", node_info.lowest_version);
            if !node_info.extra.is_empty() {
                println!("Extra: {:?}", node_info.extra);
            }
        }
        Err(e) => {
            eprintln!("Error looking up node '{}': {}", node_name, e);
            std::process::exit(1);
        }
    }

    Ok(())
}
