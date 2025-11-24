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

use crate::errors::Result;
use crate::mailbox::Message;
use crate::process::Process;
use crate::registry::ProcessRegistry;
use erltf::OwnedTerm;
use erltf::types::Atom;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub enum EventResult {
    Ok,
    Remove,
    SwapHandler(Box<dyn GenEventHandler>, OwnedTerm),
}

pub enum CallResult {
    Reply(OwnedTerm),
    Remove(OwnedTerm),
    SwapHandler(Box<dyn GenEventHandler>, OwnedTerm, OwnedTerm),
}

pub trait GenEventHandler: Send + 'static {
    fn init<'a>(
        &'a mut self,
        args: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

    fn handle_event<'a>(
        &'a mut self,
        event: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<EventResult>> + Send + 'a>>;

    fn handle_call<'a>(
        &'a mut self,
        request: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<CallResult>> + Send + 'a>>;

    fn handle_info<'a>(
        &'a mut self,
        msg: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = Result<EventResult>> + Send + 'a>> {
        let _ = msg;
        Box::pin(async move { Ok(EventResult::Ok) })
    }

    fn terminate<'a>(
        &'a mut self,
        reason: OwnedTerm,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        let _ = reason;
        Box::pin(async move {})
    }

    fn id(&self) -> OwnedTerm;
}

struct HandlerEntry {
    handler: Box<dyn GenEventHandler>,
}

pub struct GenEventManager {
    handlers: HashMap<String, HandlerEntry>,
    notify_tag: Atom,
    sync_notify_tag: Atom,
    call_tag: Atom,
    which_handlers_tag: Atom,
    registry: Arc<ProcessRegistry>,
}

impl GenEventManager {
    pub fn new(registry: Arc<ProcessRegistry>) -> Self {
        Self {
            handlers: HashMap::new(),
            notify_tag: Atom::new("$gen_notify"),
            sync_notify_tag: Atom::new("$gen_sync_notify"),
            call_tag: Atom::new("$gen_call"),
            which_handlers_tag: Atom::new("$gen_which_handlers"),
            registry,
        }
    }

    pub async fn add_handler(
        &mut self,
        handler: Box<dyn GenEventHandler>,
        args: OwnedTerm,
    ) -> Result<()> {
        let handler_id = handler.id();
        let key = format!("{:?}", handler_id);

        let mut entry = HandlerEntry { handler };
        entry.handler.init(args).await?;

        self.handlers.insert(key, entry);
        Ok(())
    }

    pub async fn delete_handler(&mut self, handler_id: OwnedTerm) -> Result<()> {
        let key = format!("{:?}", handler_id);

        if let Some(mut entry) = self.handlers.remove(&key) {
            entry
                .handler
                .terminate(OwnedTerm::Atom(Atom::new("normal")))
                .await;
            Ok(())
        } else {
            Err(crate::errors::Error::InvalidMessage(
                "Handler not found".to_string(),
            ))
        }
    }

    async fn notify(&mut self, event: OwnedTerm) -> Result<()> {
        let mut to_remove = Vec::new();

        for (key, entry) in &mut self.handlers {
            match entry.handler.handle_event(event.clone()).await {
                Ok(EventResult::Ok) => {}
                Ok(EventResult::Remove) => {
                    to_remove.push(key.clone());
                }
                Ok(EventResult::SwapHandler(new_handler, swap_args)) => {
                    entry
                        .handler
                        .terminate(OwnedTerm::Atom(Atom::new("swap")))
                        .await;
                    entry.handler = new_handler;
                    if let Err(e) = entry.handler.init(swap_args).await {
                        tracing::error!("Handler swap init failed: {}", e);
                        to_remove.push(key.clone());
                    }
                }
                Err(e) => {
                    tracing::error!("Handler {} event failed: {}", key, e);
                    to_remove.push(key.clone());
                }
            }
        }

        for key in to_remove {
            if let Some(mut entry) = self.handlers.remove(&key) {
                entry
                    .handler
                    .terminate(OwnedTerm::Atom(Atom::new("error")))
                    .await;
            }
        }

        Ok(())
    }

    async fn call_handler(
        &mut self,
        handler_id: OwnedTerm,
        request: OwnedTerm,
    ) -> Result<OwnedTerm> {
        let key = format!("{:?}", handler_id);

        if let Some(entry) = self.handlers.get_mut(&key) {
            match entry.handler.handle_call(request).await {
                Ok(CallResult::Reply(reply)) => Ok(reply),
                Ok(CallResult::Remove(reply)) => {
                    if let Some(mut removed_entry) = self.handlers.remove(&key) {
                        removed_entry
                            .handler
                            .terminate(OwnedTerm::Atom(Atom::new("normal")))
                            .await;
                    }
                    Ok(reply)
                }
                Ok(CallResult::SwapHandler(new_handler, swap_args, reply)) => {
                    entry
                        .handler
                        .terminate(OwnedTerm::Atom(Atom::new("swap")))
                        .await;
                    entry.handler = new_handler;
                    entry.handler.init(swap_args).await?;
                    Ok(reply)
                }
                Err(e) => {
                    if let Some(mut removed_entry) = self.handlers.remove(&key) {
                        removed_entry
                            .handler
                            .terminate(OwnedTerm::Atom(Atom::new("error")))
                            .await;
                    }
                    Err(e)
                }
            }
        } else {
            Err(crate::errors::Error::InvalidMessage(
                "Handler not found".to_string(),
            ))
        }
    }

    fn which_handlers(&self) -> Vec<OwnedTerm> {
        self.handlers
            .values()
            .map(|entry| entry.handler.id())
            .collect()
    }
}

impl Process for GenEventManager {
    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Regular { from, body } => {
                if let OwnedTerm::Tuple(elements) = &body
                    && elements.len() >= 2
                    && let OwnedTerm::Atom(tag) = &elements[0]
                {
                    if tag == &self.notify_tag && elements.len() == 2 {
                        let event = elements[1].clone();
                        return self.notify(event).await;
                    } else if tag == &self.sync_notify_tag && elements.len() == 2 {
                        let event = elements[1].clone();
                        self.notify(event).await?;
                        if let Some(from_pid) = from
                            && let Some(handle) = self.registry.get(&from_pid).await
                        {
                            handle
                                .send(Message::Regular {
                                    from: None,
                                    body: OwnedTerm::Atom(Atom::new("ok")),
                                })
                                .await?;
                        }
                        return Ok(());
                    } else if tag == &self.call_tag && elements.len() == 4 {
                        if let OwnedTerm::Tuple(from_tuple) = &elements[1]
                            && from_tuple.len() == 2
                            && let OwnedTerm::Pid(from_pid) = &from_tuple[0]
                            && let OwnedTerm::Reference(reference) = &from_tuple[1]
                        {
                            let handler_id = elements[2].clone();
                            let request = elements[3].clone();

                            let reply = self
                                .call_handler(handler_id, request)
                                .await
                                .unwrap_or_else(|_| OwnedTerm::Atom(Atom::new("error")));

                            let reply_msg = OwnedTerm::Tuple(vec![
                                OwnedTerm::Reference(reference.clone()),
                                reply,
                            ]);

                            if let Some(handle) = self.registry.get(from_pid).await {
                                handle
                                    .send(Message::Regular {
                                        from: None,
                                        body: reply_msg,
                                    })
                                    .await?;
                            }
                            return Ok(());
                        }
                    } else if tag == &self.which_handlers_tag
                        && elements.len() == 2
                        && let OwnedTerm::Tuple(from_tuple) = &elements[1]
                        && from_tuple.len() == 2
                        && let OwnedTerm::Pid(from_pid) = &from_tuple[0]
                        && let OwnedTerm::Reference(reference) = &from_tuple[1]
                    {
                        let handlers = self.which_handlers();
                        let reply_msg = OwnedTerm::Tuple(vec![
                            OwnedTerm::Reference(reference.clone()),
                            OwnedTerm::List(handlers),
                        ]);

                        if let Some(handle) = self.registry.get(from_pid).await {
                            handle
                                .send(Message::Regular {
                                    from: None,
                                    body: reply_msg,
                                })
                                .await?;
                        }
                        return Ok(());
                    }
                }

                for entry in self.handlers.values_mut() {
                    if let Err(e) = entry.handler.handle_info(body.clone()).await {
                        tracing::error!("Handler info failed: {}", e);
                    }
                }

                Ok(())
            }
            Message::Control { .. } => Ok(()),
            Message::Exit { reason, .. } => {
                for entry in self.handlers.values_mut() {
                    entry.handler.terminate(reason.clone()).await;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn terminate(&mut self) {
        for entry in self.handlers.values_mut() {
            entry
                .handler
                .terminate(OwnedTerm::Atom(Atom::new("shutdown")))
                .await;
        }
    }
}
