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

use edp_client::control::ControlMessage;
use erltf::OwnedTerm;
use erltf::types::{Atom, ExternalPid, ExternalReference};

pub struct ControlMessageBuilder;

impl ControlMessageBuilder {
    pub fn make_pid(node: &str, pid: u32, serial: u32, creation: u32) -> OwnedTerm {
        OwnedTerm::Pid(ExternalPid::new(Atom::new(node), pid, serial, creation))
    }

    pub fn link(from_pid: OwnedTerm, to_pid: OwnedTerm) -> ControlMessage {
        ControlMessage::Link { from_pid, to_pid }
    }

    #[allow(dead_code)]
    pub fn send(to_pid: OwnedTerm) -> ControlMessage {
        ControlMessage::Send {
            cookie: OwnedTerm::atom(""),
            to_pid,
        }
    }

    pub fn exit(from_pid: OwnedTerm, to_pid: OwnedTerm, reason: OwnedTerm) -> ControlMessage {
        ControlMessage::Exit {
            from_pid,
            to_pid,
            reason,
        }
    }

    pub fn reg_send(from_pid: OwnedTerm, to_name: &str) -> ControlMessage {
        ControlMessage::RegSend {
            from_pid,
            cookie: OwnedTerm::atom(""),
            to_name: OwnedTerm::atom(to_name),
        }
    }

    pub fn monitor_p(
        from_pid: OwnedTerm,
        to_proc: OwnedTerm,
        reference: ExternalReference,
    ) -> ControlMessage {
        ControlMessage::MonitorP {
            from_pid,
            to_proc,
            reference: OwnedTerm::Reference(reference),
        }
    }

    #[allow(dead_code)]
    pub fn demonitor_p(
        from_pid: OwnedTerm,
        to_proc: OwnedTerm,
        reference: ExternalReference,
    ) -> ControlMessage {
        ControlMessage::DemonitorP {
            from_pid,
            to_proc,
            reference: OwnedTerm::Reference(reference),
        }
    }

    #[allow(dead_code)]
    pub fn monitor_p_exit(
        from_proc: OwnedTerm,
        to_pid: OwnedTerm,
        reference: ExternalReference,
        reason: OwnedTerm,
    ) -> ControlMessage {
        ControlMessage::MonitorPExit {
            from_proc,
            to_pid,
            reference: OwnedTerm::Reference(reference),
            reason,
        }
    }

    pub fn send_sender(from_pid: OwnedTerm, to_pid: OwnedTerm) -> ControlMessage {
        ControlMessage::SendSender { from_pid, to_pid }
    }

    pub fn unlink_id(id: u64, from_pid: OwnedTerm, to_pid: OwnedTerm) -> ControlMessage {
        ControlMessage::UnlinkId {
            id,
            from_pid,
            to_pid,
        }
    }

    #[allow(dead_code)]
    pub fn unlink_id_ack(id: u64, from_pid: OwnedTerm, to_pid: OwnedTerm) -> ControlMessage {
        ControlMessage::UnlinkIdAck {
            id,
            from_pid,
            to_pid,
        }
    }

    pub fn alias_send(from_pid: OwnedTerm, alias: ExternalReference) -> ControlMessage {
        ControlMessage::AliasSend {
            from_pid,
            alias: OwnedTerm::Reference(alias),
        }
    }

    pub fn exit2(from_pid: OwnedTerm, to_pid: OwnedTerm, reason: OwnedTerm) -> ControlMessage {
        ControlMessage::Exit2 {
            from_pid,
            to_pid,
            reason,
        }
    }

    pub fn node_link() -> ControlMessage {
        ControlMessage::NodeLink
    }

    pub fn group_leader(from_pid: OwnedTerm, to_pid: OwnedTerm) -> ControlMessage {
        ControlMessage::GroupLeader { from_pid, to_pid }
    }
}
