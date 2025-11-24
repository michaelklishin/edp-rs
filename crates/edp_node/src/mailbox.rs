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

use crate::errors::{Error, Result};
use edp_client::control::ControlMessage;
use erltf::OwnedTerm;
use erltf::types::ExternalPid;
use tokio::sync::mpsc;

const DEFAULT_MAILBOX_CAPACITY: usize = 1000;

#[derive(Debug, Clone)]
pub enum Message {
    Regular {
        from: Option<ExternalPid>,
        body: OwnedTerm,
    },
    Control {
        control: Box<ControlMessage>,
        body: Option<OwnedTerm>,
    },
    Exit {
        from: ExternalPid,
        reason: OwnedTerm,
    },
    MonitorExit {
        monitored: ExternalPid,
        reference: erltf::types::ExternalReference,
        reason: OwnedTerm,
    },
    Link {
        from: ExternalPid,
    },
    Unlink {
        from: ExternalPid,
        id: u64,
    },
    Monitor {
        from: ExternalPid,
        reference: OwnedTerm,
    },
    Demonitor {
        from: ExternalPid,
        reference: OwnedTerm,
    },
}

pub struct Mailbox {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
}

impl Mailbox {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(DEFAULT_MAILBOX_CAPACITY);
        Self { sender, receiver }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self { sender, receiver }
    }

    pub fn sender(&self) -> mpsc::Sender<Message> {
        self.sender.clone()
    }

    pub async fn recv(&mut self) -> Result<Message> {
        self.receiver.recv().await.ok_or(Error::MailboxClosed)
    }

    pub fn try_recv(&mut self) -> Result<Message> {
        self.receiver.try_recv().map_err(|_| Error::MailboxClosed)
    }

    pub async fn send(&self, msg: Message) -> Result<()> {
        self.sender
            .send(msg)
            .await
            .map_err(|_| Error::MailboxClosed)
    }
}

impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}
