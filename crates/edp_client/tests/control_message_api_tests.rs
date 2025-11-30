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

mod test_control_message_builders;

use edp_client::control::ControlMessage;
use erltf::OwnedTerm;
use erltf::types::{Atom, ExternalPid, ExternalReference};
use test_control_message_builders::ControlMessageBuilder;

fn make_pid(id: u32) -> OwnedTerm {
    OwnedTerm::Pid(ExternalPid::new(Atom::new("nonode@nohost"), id, 0, 0))
}

fn make_reference() -> OwnedTerm {
    OwnedTerm::Reference(ExternalReference::new(
        Atom::new("nonode@nohost"),
        1,
        vec![1, 2, 3],
    ))
}

//
// Helper Method Tests
//

#[test]
fn test_link_helper() {
    let msg = ControlMessage::link(make_pid(1), make_pid(2));
    match msg {
        ControlMessage::Link { .. } => {}
        _ => panic!("Expected Link variant"),
    }
}

#[test]
fn test_unlink_helper() {
    let msg = ControlMessage::unlink(make_pid(1), make_pid(2));
    match msg {
        ControlMessage::Unlink { .. } => {}
        _ => panic!("Expected Unlink variant"),
    }
}

#[test]
fn test_send_helper() {
    let msg = ControlMessage::send(OwnedTerm::Atom(Atom::new("")), make_pid(1));
    match msg {
        ControlMessage::Send { .. } => {}
        _ => panic!("Expected Send variant"),
    }
}

#[test]
fn test_exit_helper() {
    let msg = ControlMessage::exit(
        make_pid(1),
        make_pid(2),
        OwnedTerm::Atom(Atom::new("normal")),
    );
    match msg {
        ControlMessage::Exit { .. } => {}
        _ => panic!("Expected Exit variant"),
    }
}

#[test]
fn test_exit2_helper() {
    let msg = ControlMessage::exit2(
        make_pid(1),
        make_pid(2),
        OwnedTerm::Atom(Atom::new("killed")),
    );
    match msg {
        ControlMessage::Exit2 { .. } => {}
        _ => panic!("Expected Exit2 variant"),
    }
}

#[test]
fn test_reg_send_helper() {
    let msg = ControlMessage::reg_send(
        make_pid(1),
        OwnedTerm::Atom(Atom::new("")),
        OwnedTerm::Atom(Atom::new("test")),
    );
    match msg {
        ControlMessage::RegSend { .. } => {}
        _ => panic!("Expected RegSend variant"),
    }
}

#[test]
fn test_group_leader_helper() {
    let msg = ControlMessage::group_leader(make_pid(1), make_pid(2));
    match msg {
        ControlMessage::GroupLeader { .. } => {}
        _ => panic!("Expected GroupLeader variant"),
    }
}

#[test]
fn test_send_sender_helper() {
    let msg = ControlMessage::send_sender(make_pid(1), make_pid(2));
    match msg {
        ControlMessage::SendSender { .. } => {}
        _ => panic!("Expected SendSender variant"),
    }
}

#[test]
fn test_monitor_p_helper() {
    let msg = ControlMessage::monitor_p(
        make_pid(1),
        OwnedTerm::Atom(Atom::new("test")),
        make_reference(),
    );
    match msg {
        ControlMessage::MonitorP { .. } => {}
        _ => panic!("Expected MonitorP variant"),
    }
}

#[test]
fn test_demonitor_p_helper() {
    let msg = ControlMessage::demonitor_p(
        make_pid(1),
        OwnedTerm::Atom(Atom::new("test")),
        make_reference(),
    );
    match msg {
        ControlMessage::DemonitorP { .. } => {}
        _ => panic!("Expected DemonitorP variant"),
    }
}

#[test]
fn test_monitor_p_exit_helper() {
    let msg = ControlMessage::monitor_p_exit(
        make_pid(1),
        make_pid(2),
        make_reference(),
        OwnedTerm::Atom(Atom::new("noproc")),
    );
    match msg {
        ControlMessage::MonitorPExit { .. } => {}
        _ => panic!("Expected MonitorPExit variant"),
    }
}

#[test]
fn test_payload_exit_helper() {
    let msg = ControlMessage::payload_exit(make_pid(1), make_pid(2));
    match msg {
        ControlMessage::PayloadExit { .. } => {}
        _ => panic!("Expected PayloadExit variant"),
    }
}

#[test]
fn test_payload_exit2_helper() {
    let msg = ControlMessage::payload_exit2(make_pid(1), make_pid(2));
    match msg {
        ControlMessage::PayloadExit2 { .. } => {}
        _ => panic!("Expected PayloadExit2 variant"),
    }
}

#[test]
fn test_payload_monitor_p_exit_helper() {
    let msg = ControlMessage::payload_monitor_p_exit(make_pid(1), make_pid(2), make_reference());
    match msg {
        ControlMessage::PayloadMonitorPExit { .. } => {}
        _ => panic!("Expected PayloadMonitorPExit variant"),
    }
}

//
// Builder Tests
//

#[test]
fn test_builder_creates_valid_link_message() {
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let to = ControlMessageBuilder::make_pid("test@localhost", 1, 0, 1);

    let msg = ControlMessageBuilder::link(from.clone(), to.clone());

    match msg {
        ControlMessage::Link { from_pid, to_pid } => {
            assert_eq!(from_pid, from);
            assert_eq!(to_pid, to);
        }
        _ => panic!("Expected Link message"),
    }
}

#[test]
fn test_builder_creates_exit_message_with_reason() {
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let to = ControlMessageBuilder::make_pid("test@localhost", 1, 0, 1);
    let reason = OwnedTerm::atom("shutdown");

    let msg = ControlMessageBuilder::exit(from.clone(), to.clone(), reason.clone());

    match msg {
        ControlMessage::Exit {
            from_pid,
            to_pid,
            reason: r,
        } => {
            assert_eq!(from_pid, from);
            assert_eq!(to_pid, to);
            assert_eq!(r, reason);
        }
        _ => panic!("Expected Exit message"),
    }
}

#[test]
fn test_builder_creates_reg_send_message_with_atom() {
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let msg = ControlMessageBuilder::reg_send(from.clone(), "my_server");

    match msg {
        ControlMessage::RegSend {
            from_pid, to_name, ..
        } => {
            assert_eq!(from_pid, from);
            assert_eq!(to_name, OwnedTerm::atom("my_server"));
        }
        _ => panic!("Expected RegSend message"),
    }
}

#[test]
fn test_builder_creates_monitor_p_message() {
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let to_proc = ControlMessageBuilder::make_pid("other@host", 5, 0, 1);
    let reference = ExternalReference::new(Atom::new("test@localhost"), 1, vec![0, 0, 0]);

    let msg = ControlMessageBuilder::monitor_p(from.clone(), to_proc.clone(), reference.clone());

    match msg {
        ControlMessage::MonitorP {
            from_pid,
            to_proc: proc,
            reference: ref_val,
        } => {
            assert_eq!(from_pid, from);
            assert_eq!(proc, to_proc);
            assert_eq!(ref_val, OwnedTerm::Reference(reference));
        }
        _ => panic!("Expected MonitorP message"),
    }
}

#[test]
fn test_builder_creates_send_sender_message_with_explicit_sender() {
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let to = ControlMessageBuilder::make_pid("test@localhost", 1, 0, 1);

    let msg = ControlMessageBuilder::send_sender(from.clone(), to.clone());

    match msg {
        ControlMessage::SendSender { from_pid, to_pid } => {
            assert_eq!(from_pid, from);
            assert_eq!(to_pid, to);
        }
        _ => panic!("Expected SendSender message"),
    }
}

#[test]
fn test_builder_creates_unlink_id_message() {
    let id = 12345u64;
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let to = ControlMessageBuilder::make_pid("test@localhost", 1, 0, 1);

    let msg = ControlMessageBuilder::unlink_id(id, from.clone(), to.clone());

    match msg {
        ControlMessage::UnlinkId {
            id: msg_id,
            from_pid,
            to_pid,
        } => {
            assert_eq!(msg_id, id);
            assert_eq!(from_pid, from);
            assert_eq!(to_pid, to);
        }
        _ => panic!("Expected UnlinkId message"),
    }
}

#[test]
fn test_builder_creates_alias_send_message() {
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let alias = ExternalReference::new(Atom::new("test@localhost"), 2, vec![1, 2, 3]);

    let msg = ControlMessageBuilder::alias_send(from.clone(), alias.clone());

    match msg {
        ControlMessage::AliasSend {
            from_pid,
            alias: alias_ref,
        } => {
            assert_eq!(from_pid, from);
            assert_eq!(alias_ref, OwnedTerm::Reference(alias));
        }
        _ => panic!("Expected AliasSend message"),
    }
}

#[test]
fn test_builder_creates_node_link_message() {
    let msg = ControlMessageBuilder::node_link();

    match msg {
        ControlMessage::NodeLink => {}
        _ => panic!("Expected NodeLink message"),
    }
}

#[test]
fn test_builder_creates_group_leader_message() {
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let to = ControlMessageBuilder::make_pid("test@localhost", 1, 0, 1);

    let msg = ControlMessageBuilder::group_leader(from.clone(), to.clone());

    match msg {
        ControlMessage::GroupLeader { from_pid, to_pid } => {
            assert_eq!(from_pid, from);
            assert_eq!(to_pid, to);
        }
        _ => panic!("Expected GroupLeader message"),
    }
}

#[test]
fn test_builder_creates_exit2_message() {
    let from = ControlMessageBuilder::make_pid("test@localhost", 0, 0, 1);
    let to = ControlMessageBuilder::make_pid("test@localhost", 1, 0, 1);
    let reason = OwnedTerm::atom("kill");

    let msg = ControlMessageBuilder::exit2(from.clone(), to.clone(), reason.clone());

    match msg {
        ControlMessage::Exit2 {
            from_pid,
            to_pid,
            reason: r,
        } => {
            assert_eq!(from_pid, from);
            assert_eq!(to_pid, to);
            assert_eq!(r, reason);
        }
        _ => panic!("Expected Exit2 message"),
    }
}
