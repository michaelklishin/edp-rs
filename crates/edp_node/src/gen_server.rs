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
use erltf::types::{Atom, ExternalPid, ExternalReference};
use std::future::Future;
use std::sync::Arc;

pub enum CallResult {
    Reply(OwnedTerm),
    NoReply,
}

pub trait GenServer: Send + 'static {
    fn init(&mut self, args: Vec<OwnedTerm>) -> impl Future<Output = Result<()>> + Send + '_;

    fn handle_call(
        &mut self,
        msg: OwnedTerm,
        from: ExternalPid,
    ) -> impl Future<Output = Result<CallResult>> + Send + '_;

    fn handle_cast(&mut self, msg: OwnedTerm) -> impl Future<Output = Result<()>> + Send + '_;

    fn handle_info(&mut self, msg: OwnedTerm) -> impl Future<Output = Result<()>> + Send + '_;

    fn terminate(&mut self, _reason: OwnedTerm) -> impl Future<Output = ()> + Send + '_ {
        async move {}
    }
}

pub struct GenServerProcess<T: GenServer> {
    server: T,
    call_tag: Atom,
    cast_tag: Atom,
    registry: Arc<ProcessRegistry>,
}

impl<T: GenServer> GenServerProcess<T> {
    pub fn new(server: T, registry: Arc<ProcessRegistry>) -> Self {
        Self {
            server,
            call_tag: Atom::new("$gen_call"),
            cast_tag: Atom::new("$gen_cast"),
            registry,
        }
    }

    async fn handle_gen_call(
        &mut self,
        from_pid: ExternalPid,
        reference: ExternalReference,
        request: OwnedTerm,
    ) -> Result<()> {
        let result = self.server.handle_call(request, from_pid.clone()).await?;

        match result {
            CallResult::Reply(reply) => {
                let reply_msg = OwnedTerm::Tuple(vec![OwnedTerm::Reference(reference), reply]);

                if let Some(handle) = self.registry.get(&from_pid).await {
                    handle
                        .send(Message::Regular {
                            from: None,
                            body: reply_msg,
                        })
                        .await?;
                }

                Ok(())
            }
            CallResult::NoReply => Ok(()),
        }
    }

    async fn handle_gen_cast(&mut self, request: OwnedTerm) -> Result<()> {
        self.server.handle_cast(request).await
    }
}

impl<T: GenServer> Process for GenServerProcess<T> {
    async fn handle_message(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Regular { from: _, body } => {
                if let OwnedTerm::Tuple(elements) = &body
                    && elements.len() >= 2
                    && let OwnedTerm::Atom(tag) = &elements[0]
                {
                    if tag == &self.call_tag && elements.len() == 3 {
                        if let OwnedTerm::Tuple(from_tuple) = &elements[1]
                            && from_tuple.len() == 2
                            && let OwnedTerm::Pid(from_pid) = &from_tuple[0]
                            && let OwnedTerm::Reference(reference) = &from_tuple[1]
                        {
                            let request = elements[2].clone();
                            return self
                                .handle_gen_call(from_pid.clone(), reference.clone(), request)
                                .await;
                        }
                    } else if tag == &self.cast_tag && elements.len() == 2 {
                        return self.handle_gen_cast(elements[1].clone()).await;
                    }
                }

                self.server.handle_info(body).await
            }
            Message::Control { .. } => Ok(()),
            Message::Exit { reason, .. } => {
                self.server.terminate(reason).await;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    async fn terminate(&mut self) {
        self.server
            .terminate(OwnedTerm::Atom(Atom::new("normal")))
            .await;
    }
}
