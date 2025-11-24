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

#[tokio::main]
async fn main() -> Result<()> {
    let epmd = EpmdClient::new("localhost");

    match epmd.list_nodes().await {
        Ok(names) => {
            if names.trim().is_empty() {
                println!("No nodes registered with EPMD");
            } else {
                for line in names.lines() {
                    println!("{}", line);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to query EPMD: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
