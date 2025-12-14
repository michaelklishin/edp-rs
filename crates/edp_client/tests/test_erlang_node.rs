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

use edp_client::epmd_client::EpmdClient;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

pub const TEST_COOKIE: &str = "test-cookie";

pub struct TestErlangNode {
    child: Child,
}

impl TestErlangNode {
    pub async fn new(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let child = Command::new("erl")
            .args(["-sname", name, "-noshell", "-setcookie", TEST_COOKIE])
            .spawn()?;

        let start = Instant::now();
        let mut registered = false;
        loop {
            match try_epmd_client().await {
                Ok(client) => match client.lookup_node(name).await {
                    Ok(_) => {
                        registered = true;
                        break;
                    }
                    Err(_) => {
                        if start.elapsed() > Duration::from_secs(10) {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                },
                Err(_) => {
                    if start.elapsed() > Duration::from_secs(10) {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }

        if registered {
            Ok(Self { child })
        } else {
            let node = Self { child };
            drop(node);
            Err("Erlang node failed to register with EPMD within timeout".into())
        }
    }
}

impl Drop for TestErlangNode {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub async fn try_epmd_client() -> Result<EpmdClient, Box<dyn std::error::Error>> {
    let client = EpmdClient::new("127.0.0.1");
    Ok(client)
}

pub async fn epmd_client() -> EpmdClient {
    try_epmd_client().await.unwrap()
}
