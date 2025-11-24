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

use edp_node::{CallResult, GenServer, GenServerProcess, Node, Result};
use erltf::OwnedTerm;
use erltf::types::{Atom, ExternalPid};
use std::sync::Arc;
use tokio::sync::Mutex;

struct CounterServer {
    count: i64,
    casts_received: Arc<Mutex<Vec<OwnedTerm>>>,
}

impl CounterServer {
    fn new(casts_received: Arc<Mutex<Vec<OwnedTerm>>>) -> Self {
        Self {
            count: 0,
            casts_received,
        }
    }
}

impl GenServer for CounterServer {
    async fn init(&mut self, _args: Vec<OwnedTerm>) -> Result<()> {
        Ok(())
    }

    async fn handle_call(&mut self, msg: OwnedTerm, _from: ExternalPid) -> Result<CallResult> {
        if let OwnedTerm::Atom(atom) = msg {
            match atom.as_str() {
                "get" => Ok(CallResult::Reply(OwnedTerm::Integer(self.count))),
                "increment" => {
                    self.count += 1;
                    Ok(CallResult::Reply(OwnedTerm::Integer(self.count)))
                }
                _ => Ok(CallResult::NoReply),
            }
        } else {
            Ok(CallResult::NoReply)
        }
    }

    async fn handle_cast(&mut self, msg: OwnedTerm) -> Result<()> {
        self.casts_received.lock().await.push(msg.clone());
        if let OwnedTerm::Atom(atom) = msg
            && atom.as_str() == "increment"
        {
            self.count += 1;
        }
        Ok(())
    }

    async fn handle_info(&mut self, _msg: OwnedTerm) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_genserver_call() {
    let mut node = Node::new("test_genserver@localhost", "secret");
    node.start(5558).await.unwrap();

    let casts = Arc::new(Mutex::new(Vec::new()));
    let server = CounterServer::new(casts.clone());
    let process = GenServerProcess::new(server, node.registry());

    let pid = node.spawn(process).await.unwrap();
    node.register(Atom::new("counter"), pid.clone())
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_genserver_cast() {
    let mut node = Node::new("test_genserver_cast@localhost", "secret");
    node.start(5559).await.unwrap();

    let casts = Arc::new(Mutex::new(Vec::new()));
    let server = CounterServer::new(casts.clone());
    let process = GenServerProcess::new(server, node.registry());

    let pid = node.spawn(process).await.unwrap();

    let cast_msg = OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("$gen_cast")),
        OwnedTerm::Atom(Atom::new("increment")),
    ]);

    node.send(&pid, cast_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let received_casts = casts.lock().await;
    assert_eq!(received_casts.len(), 1);
    assert_eq!(received_casts[0], OwnedTerm::Atom(Atom::new("increment")));
}
