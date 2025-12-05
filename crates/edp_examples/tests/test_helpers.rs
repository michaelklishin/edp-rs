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
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;
use tokio::time::sleep;

const DEFAULT_COOKIE: &str = "monster";
const NODE_START_DELAY_MS: u64 = 500;
const VERIFICATION_ATTEMPTS: u32 = 10;
const VERIFICATION_DELAY_MS: u64 = 100;
const STARTUP_RETRY_ATTEMPTS: u32 = 3;
const RETRY_DELAY_MS: u64 = 500;

pub struct TestNode {
    _child: Child,
    _temp_dir: PathBuf,
    node_name: String,
    hostname: String,
}

impl TestNode {
    pub fn start(short_name: &str) -> Result<Self> {
        for attempt in 1..=STARTUP_RETRY_ATTEMPTS {
            match Self::try_start(short_name) {
                Ok(node) => {
                    if Self::verify_node_registered(short_name, attempt) {
                        return Ok(node);
                    }
                }
                Err(e) if attempt == STARTUP_RETRY_ATTEMPTS => return Err(e),
                Err(_) => {
                    thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
                    continue;
                }
            }
        }
        Err(anyhow::anyhow!(
            "Failed to start and verify node after {} attempts",
            STARTUP_RETRY_ATTEMPTS
        ))
    }

    fn try_start(short_name: &str) -> Result<Self> {
        let temp_dir =
            env::temp_dir().join(format!("edp_test_{}_{}", short_name, std::process::id()));
        fs::create_dir_all(&temp_dir)?;

        let source_module = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("erl")
            .join("test_node.erl");

        let dest_module = temp_dir.join("test_node.erl");
        fs::copy(&source_module, &dest_module)?;

        Command::new("erlc")
            .args([
                "-o",
                temp_dir.to_str().unwrap(),
                dest_module.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to compile test_node.erl");

        let hostname = Command::new("hostname")
            .arg("-s")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "localhost".to_string());

        let node_name_full = format!("{}@{}", short_name, &hostname);
        let start_command = format!("test_node:start('{}', '{}')", short_name, &hostname);

        let child = Command::new("erl")
            .args([
                "-sname",
                short_name,
                "-setcookie",
                DEFAULT_COOKIE,
                "-eval",
                &start_command,
                "-noshell",
                "-pa",
                temp_dir.to_str().unwrap(),
            ])
            .spawn()
            .expect("Failed to start Erlang test node");

        Ok(TestNode {
            _child: child,
            _temp_dir: temp_dir,
            node_name: node_name_full,
            hostname,
        })
    }

    fn verify_node_registered(short_name: &str, attempt: u32) -> bool {
        for _ in 0..VERIFICATION_ATTEMPTS {
            thread::sleep(Duration::from_millis(VERIFICATION_DELAY_MS));
            if let Ok(output) = Command::new("epmd").arg("-names").output() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if stdout.contains(&format!("name {} at port", short_name)) {
                        return true;
                    }
                }
            }
        }
        eprintln!(
            "Node {} not registered in epmd after attempt {}",
            short_name, attempt
        );
        false
    }

    pub fn name(&self) -> &str {
        &self.node_name
    }

    pub fn client_node_name(&self, client_short_name: &str) -> String {
        format!("{}@{}", client_short_name, self.hostname)
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        let _ = self._child.kill();
        let _ = fs::remove_dir_all(&self._temp_dir);
    }
}

pub struct TestContext {
    pub node: Node,
    pub target_node_name: String,
    pub _test_node: TestNode,
}

impl TestContext {
    pub async fn new(test_name: &str) -> Result<Self> {
        let test_node = TestNode::start(&format!("erl_test_{}", test_name))?;
        sleep(Duration::from_millis(NODE_START_DELAY_MS)).await;

        let target_node_name = test_node.name().to_string();
        let client_node_name = test_node.client_node_name(&format!("rust_test_{}", test_name));

        let mut node = Node::new(client_node_name, DEFAULT_COOKIE.to_string());
        node.start(0).await?;
        node.connect(&target_node_name).await?;

        Ok(TestContext {
            node,
            target_node_name,
            _test_node: test_node,
        })
    }

    pub async fn rpc_call(
        &mut self,
        module: &str,
        function: &str,
        args: Vec<OwnedTerm>,
    ) -> Result<OwnedTerm> {
        Ok(self
            .node
            .rpc_call_raw(&self.target_node_name, module, function, args)
            .await?)
    }

    #[allow(dead_code)]
    pub async fn rpc_call_unwrapped(
        &mut self,
        module: &str,
        function: &str,
        args: Vec<OwnedTerm>,
    ) -> Result<OwnedTerm> {
        Ok(self
            .node
            .rpc_call(&self.target_node_name, module, function, args)
            .await?)
    }

    pub fn unwrap_rex_response(response: OwnedTerm) -> Result<OwnedTerm> {
        match response {
            OwnedTerm::Tuple(elements) if elements.len() == 2 => {
                if let OwnedTerm::Atom(a) = &elements[0] {
                    if a == "rex" {
                        return Ok(elements[1].clone());
                    }
                }
                Err(anyhow::anyhow!("Expected rex tuple, got {:?}", elements[0]))
            }
            _ => Err(anyhow::anyhow!(
                "Expected tuple response, got {:?}",
                response
            )),
        }
    }
}
