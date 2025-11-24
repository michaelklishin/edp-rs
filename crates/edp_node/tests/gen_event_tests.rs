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

use edp_node::{EventResult, GenEventCallResult, GenEventHandler, GenEventManager, Node, Result};
use erltf::OwnedTerm;
use erltf::types::Atom;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

struct LoggerHandler {
    id: OwnedTerm,
    events: Arc<Mutex<Vec<OwnedTerm>>>,
}

impl LoggerHandler {
    fn new(id: OwnedTerm, events: Arc<Mutex<Vec<OwnedTerm>>>) -> Self {
        Self { id, events }
    }
}

impl GenEventHandler for LoggerHandler {
    fn init<'a>(
        &'a mut self,
        _args: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn handle_event<'a>(
        &'a mut self,
        event: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<EventResult>> + Send + 'a>> {
        Box::pin(async move {
            self.events.lock().await.push(event);
            Ok(EventResult::Ok)
        })
    }

    fn handle_call<'a>(
        &'a mut self,
        request: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<GenEventCallResult>> + Send + 'a>> {
        Box::pin(async move {
            if let OwnedTerm::Atom(atom) = &request
                && atom.as_str() == "get_count"
            {
                let count = self.events.lock().await.len() as i64;
                return Ok(GenEventCallResult::Reply(OwnedTerm::Integer(count)));
            }
            Ok(GenEventCallResult::Reply(OwnedTerm::Atom(Atom::new(
                "unknown",
            ))))
        })
    }

    fn id(&self) -> OwnedTerm {
        self.id.clone()
    }
}

struct RemovingHandler {
    id: OwnedTerm,
    remove_on: String,
}

impl RemovingHandler {
    fn new(id: OwnedTerm, remove_on: String) -> Self {
        Self { id, remove_on }
    }
}

impl GenEventHandler for RemovingHandler {
    fn init<'a>(
        &'a mut self,
        _args: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }

    fn handle_event<'a>(
        &'a mut self,
        event: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<EventResult>> + Send + 'a>> {
        Box::pin(async move {
            if let OwnedTerm::Atom(atom) = &event
                && atom.as_str() == self.remove_on
            {
                return Ok(EventResult::Remove);
            }
            Ok(EventResult::Ok)
        })
    }

    fn handle_call<'a>(
        &'a mut self,
        _request: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<GenEventCallResult>> + Send + 'a>> {
        Box::pin(async move { Ok(GenEventCallResult::Reply(OwnedTerm::Atom(Atom::new("ok")))) })
    }

    fn id(&self) -> OwnedTerm {
        self.id.clone()
    }
}

//
// Event Notification Tests
//

#[tokio::test]
async fn test_gen_event_notify() {
    let mut node = Node::new("test_genevent@localhost", "secret");
    node.start(5580).await.unwrap();

    let events = Arc::new(Mutex::new(Vec::new()));
    let handler = LoggerHandler::new(OwnedTerm::Atom(Atom::new("logger")), events.clone());

    let mut manager = GenEventManager::new(node.registry());
    manager
        .add_handler(Box::new(handler), OwnedTerm::Atom(Atom::new("ok")))
        .await
        .unwrap();

    let pid = node.spawn(manager).await.unwrap();

    let notify_msg = OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("$gen_notify")),
        OwnedTerm::Atom(Atom::new("test_event")),
    ]);
    node.send(&pid, notify_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let collected = events.lock().await;
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0], OwnedTerm::Atom(Atom::new("test_event")));
}

#[tokio::test]
async fn test_gen_event_multiple_handlers() {
    let mut node = Node::new("test_genevent_multi@localhost", "secret");
    node.start(5581).await.unwrap();

    let events1 = Arc::new(Mutex::new(Vec::new()));
    let events2 = Arc::new(Mutex::new(Vec::new()));

    let handler1 = LoggerHandler::new(OwnedTerm::Atom(Atom::new("logger1")), events1.clone());
    let handler2 = LoggerHandler::new(OwnedTerm::Atom(Atom::new("logger2")), events2.clone());

    let mut manager = GenEventManager::new(node.registry());
    manager
        .add_handler(Box::new(handler1), OwnedTerm::Atom(Atom::new("ok")))
        .await
        .unwrap();
    manager
        .add_handler(Box::new(handler2), OwnedTerm::Atom(Atom::new("ok")))
        .await
        .unwrap();

    let pid = node.spawn(manager).await.unwrap();

    let notify_msg = OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("$gen_notify")),
        OwnedTerm::Integer(42),
    ]);
    node.send(&pid, notify_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let collected1 = events1.lock().await;
    let collected2 = events2.lock().await;

    assert_eq!(collected1.len(), 1);
    assert_eq!(collected2.len(), 1);
    assert_eq!(collected1[0], OwnedTerm::Integer(42));
    assert_eq!(collected2[0], OwnedTerm::Integer(42));
}

//
// Handler Management Tests
//

#[tokio::test]
async fn test_gen_event_handler_removal() {
    let mut node = Node::new("test_genevent_remove@localhost", "secret");
    node.start(5582).await.unwrap();

    let handler = RemovingHandler::new(OwnedTerm::Atom(Atom::new("removing")), "stop".to_string());

    let mut manager = GenEventManager::new(node.registry());
    manager
        .add_handler(Box::new(handler), OwnedTerm::Atom(Atom::new("ok")))
        .await
        .unwrap();

    let pid = node.spawn(manager).await.unwrap();

    let handlers = node.registry().get(&pid).await.unwrap();
    assert!(handlers.pid == pid);

    let notify_msg = OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("$gen_notify")),
        OwnedTerm::Atom(Atom::new("continue")),
    ]);
    node.send(&pid, notify_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let stop_msg = OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("$gen_notify")),
        OwnedTerm::Atom(Atom::new("stop")),
    ]);
    node.send(&pid, stop_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_gen_event_which_handlers() {
    let mut node = Node::new("test_genevent_which@localhost", "secret");
    node.start(5583).await.unwrap();

    let events1 = Arc::new(Mutex::new(Vec::new()));
    let events2 = Arc::new(Mutex::new(Vec::new()));

    let handler1 = LoggerHandler::new(OwnedTerm::Atom(Atom::new("logger1")), events1.clone());
    let handler2 = LoggerHandler::new(OwnedTerm::Atom(Atom::new("logger2")), events2.clone());

    let mut manager = GenEventManager::new(node.registry());
    manager
        .add_handler(Box::new(handler1), OwnedTerm::Atom(Atom::new("ok")))
        .await
        .unwrap();
    manager
        .add_handler(Box::new(handler2), OwnedTerm::Atom(Atom::new("ok")))
        .await
        .unwrap();

    let _pid = node.spawn(manager).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}

#[tokio::test]
async fn test_gen_event_delete_handler() {
    let mut node = Node::new("test_genevent_delete@localhost", "secret");
    node.start(5584).await.unwrap();

    let events = Arc::new(Mutex::new(Vec::new()));
    let handler = LoggerHandler::new(OwnedTerm::Atom(Atom::new("logger")), events.clone());

    let mut manager = GenEventManager::new(node.registry());
    manager
        .add_handler(Box::new(handler), OwnedTerm::Atom(Atom::new("ok")))
        .await
        .unwrap();

    manager
        .delete_handler(OwnedTerm::Atom(Atom::new("logger")))
        .await
        .unwrap();

    let pid = node.spawn(manager).await.unwrap();

    let notify_msg = OwnedTerm::Tuple(vec![
        OwnedTerm::Atom(Atom::new("$gen_notify")),
        OwnedTerm::Atom(Atom::new("test")),
    ]);
    node.send(&pid, notify_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let collected = events.lock().await;
    assert_eq!(collected.len(), 0);
}
