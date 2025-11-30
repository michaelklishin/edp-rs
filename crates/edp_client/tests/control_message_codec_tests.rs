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

fn make_pid(id: u32, serial: u32, creation: u32) -> OwnedTerm {
    OwnedTerm::Pid(ExternalPid::new(
        Atom::new("nonode@nohost"),
        id,
        serial,
        creation,
    ))
}

fn make_reference() -> OwnedTerm {
    OwnedTerm::Reference(ExternalReference::new(
        Atom::new("nonode@nohost"),
        1,
        vec![1, 2, 3],
    ))
}

//
// Basic Roundtrip Tests
//

#[test]
fn test_link_roundtrip() {
    let msg = ControlMessage::Link {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::Link { .. } => {}
        _ => panic!("Expected Link variant"),
    }
}

#[test]
fn test_send_roundtrip() {
    let msg = ControlMessage::Send {
        cookie: OwnedTerm::Atom(Atom::new("")),
        to_pid: make_pid(1, 0, 0),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::Send { .. } => {}
        _ => panic!("Expected Send variant"),
    }
}

#[test]
fn test_exit_roundtrip() {
    let msg = ControlMessage::Exit {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        reason: OwnedTerm::Atom(Atom::new("normal")),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::Exit { .. } => {}
        _ => panic!("Expected Exit variant"),
    }
}

#[test]
fn test_unlink_roundtrip() {
    let msg = ControlMessage::Unlink {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::Unlink { .. } => {}
        _ => panic!("Expected Unlink variant"),
    }
}

#[test]
fn test_node_link_roundtrip() {
    let msg = ControlMessage::NodeLink;

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::NodeLink => {}
        _ => panic!("Expected NodeLink variant"),
    }
}

#[test]
fn test_reg_send_roundtrip() {
    let msg = ControlMessage::RegSend {
        from_pid: make_pid(1, 0, 0),
        cookie: OwnedTerm::Atom(Atom::new("")),
        to_name: OwnedTerm::Atom(Atom::new("test")),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::RegSend { .. } => {}
        _ => panic!("Expected RegSend variant"),
    }
}

#[test]
fn test_group_leader_roundtrip() {
    let msg = ControlMessage::GroupLeader {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::GroupLeader { .. } => {}
        _ => panic!("Expected GroupLeader variant"),
    }
}

#[test]
fn test_exit2_roundtrip() {
    let msg = ControlMessage::Exit2 {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        reason: OwnedTerm::Atom(Atom::new("killed")),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::Exit2 { .. } => {}
        _ => panic!("Expected Exit2 variant"),
    }
}

#[test]
fn test_send_sender_roundtrip() {
    let msg = ControlMessage::SendSender {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::SendSender { .. } => {}
        _ => panic!("Expected SendSender variant"),
    }
}

#[test]
fn test_payload_exit_roundtrip() {
    let msg = ControlMessage::PayloadExit {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::PayloadExit { .. } => {}
        _ => panic!("Expected PayloadExit variant"),
    }
}

#[test]
fn test_payload_exit2_roundtrip() {
    let msg = ControlMessage::PayloadExit2 {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::PayloadExit2 { .. } => {}
        _ => panic!("Expected PayloadExit2 variant"),
    }
}

#[test]
fn test_payload_monitor_p_exit_roundtrip() {
    let msg = ControlMessage::PayloadMonitorPExit {
        from_proc: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        reference: make_reference(),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::PayloadMonitorPExit { .. } => {}
        _ => panic!("Expected PayloadMonitorPExit variant"),
    }
}

#[test]
fn test_monitor_p_roundtrip() {
    let msg = ControlMessage::MonitorP {
        from_pid: make_pid(1, 0, 0),
        to_proc: OwnedTerm::Atom(Atom::new("test")),
        reference: make_reference(),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::MonitorP { .. } => {}
        _ => panic!("Expected MonitorP variant"),
    }
}

#[test]
fn test_demonitor_p_roundtrip() {
    let msg = ControlMessage::DemonitorP {
        from_pid: make_pid(1, 0, 0),
        to_proc: OwnedTerm::Atom(Atom::new("test")),
        reference: make_reference(),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::DemonitorP { .. } => {}
        _ => panic!("Expected DemonitorP variant"),
    }
}

#[test]
fn test_monitor_p_exit_roundtrip() {
    let msg = ControlMessage::MonitorPExit {
        from_proc: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        reference: make_reference(),
        reason: OwnedTerm::Atom(Atom::new("noproc")),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::MonitorPExit { .. } => {}
        _ => panic!("Expected MonitorPExit variant"),
    }
}

//
// Trace Token Tests
//

#[test]
fn test_send_tt_roundtrip() {
    let msg = ControlMessage::SendTt {
        cookie: OwnedTerm::Atom(Atom::new("")),
        to_pid: make_pid(1, 0, 0),
        trace_token: OwnedTerm::Integer(42),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::SendTt { .. } => {}
        _ => panic!("Expected SendTt variant"),
    }
}

#[test]
fn test_exit_tt_roundtrip() {
    let msg = ControlMessage::ExitTt {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        trace_token: OwnedTerm::Integer(42),
        reason: OwnedTerm::Atom(Atom::new("normal")),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::ExitTt { .. } => {}
        _ => panic!("Expected ExitTt variant"),
    }
}

#[test]
fn test_reg_send_tt_roundtrip() {
    let msg = ControlMessage::RegSendTt {
        from_pid: make_pid(1, 0, 0),
        cookie: OwnedTerm::Atom(Atom::new("")),
        to_name: OwnedTerm::Atom(Atom::new("test")),
        trace_token: OwnedTerm::Integer(42),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::RegSendTt { .. } => {}
        _ => panic!("Expected RegSendTt variant"),
    }
}

#[test]
fn test_exit2_tt_roundtrip() {
    let msg = ControlMessage::Exit2Tt {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        trace_token: OwnedTerm::Integer(42),
        reason: OwnedTerm::Atom(Atom::new("killed")),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::Exit2Tt { .. } => {}
        _ => panic!("Expected Exit2Tt variant"),
    }
}

#[test]
fn test_send_sender_tt_roundtrip() {
    let msg = ControlMessage::SendSenderTt {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        trace_token: OwnedTerm::Integer(42),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::SendSenderTt { .. } => {}
        _ => panic!("Expected SendSenderTt variant"),
    }
}

#[test]
fn test_payload_exit_tt_roundtrip() {
    let msg = ControlMessage::PayloadExitTt {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        trace_token: OwnedTerm::Integer(42),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::PayloadExitTt { .. } => {}
        _ => panic!("Expected PayloadExitTt variant"),
    }
}

#[test]
fn test_payload_exit2_tt_roundtrip() {
    let msg = ControlMessage::PayloadExit2Tt {
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
        trace_token: OwnedTerm::Integer(42),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::PayloadExit2Tt { .. } => {}
        _ => panic!("Expected PayloadExit2Tt variant"),
    }
}

//
// Spawn Tests
//

#[test]
fn test_spawn_request_tt_roundtrip() {
    let msg = ControlMessage::SpawnRequestTt {
        req_id: make_reference(),
        from: make_pid(1, 0, 0),
        group_leader: make_pid(2, 0, 0),
        mfa: OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("erlang")),
            OwnedTerm::Atom(Atom::new("apply")),
            OwnedTerm::Integer(2),
        ]),
        arg_list: OwnedTerm::List(vec![]),
        opt_list: OwnedTerm::List(vec![]),
        trace_token: OwnedTerm::Integer(42),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::SpawnRequestTt { .. } => {}
        _ => panic!("Expected SpawnRequestTt variant"),
    }
}

#[test]
fn test_spawn_reply_tt_roundtrip() {
    let msg = ControlMessage::SpawnReplyTt {
        req_id: make_reference(),
        to: make_pid(1, 0, 0),
        flags: OwnedTerm::Integer(0),
        result: make_pid(2, 0, 0),
        trace_token: OwnedTerm::Integer(42),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::SpawnReplyTt { .. } => {}
        _ => panic!("Expected SpawnReplyTt variant"),
    }
}

#[test]
fn test_spawn_request_roundtrip() {
    let msg = ControlMessage::SpawnRequest {
        req_id: make_reference(),
        from: make_pid(1, 0, 0),
        group_leader: make_pid(2, 0, 0),
        mfa: OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("erlang")),
            OwnedTerm::Atom(Atom::new("apply")),
            OwnedTerm::Integer(2),
        ]),
        arg_list: OwnedTerm::List(vec![]),
        opt_list: OwnedTerm::List(vec![]),
    };

    let term = msg.to_term();
    let decoded = ControlMessage::from_term(&term).unwrap();

    assert_eq!(msg, decoded);
}

#[test]
fn test_spawn_reply_roundtrip() {
    let msg = ControlMessage::SpawnReply {
        req_id: make_reference(),
        to: make_pid(1, 0, 0),
        flags: OwnedTerm::Integer(0),
        result: OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("ok")), make_pid(2, 0, 0)]),
    };

    let term = msg.to_term();
    let decoded = ControlMessage::from_term(&term).unwrap();

    assert_eq!(msg, decoded);
}

//
// Alias Tests
//

#[test]
fn test_alias_send_tt_roundtrip() {
    let msg = ControlMessage::AliasSendTt {
        from_pid: make_pid(1, 0, 0),
        alias: make_reference(),
        trace_token: OwnedTerm::Integer(42),
    };

    let term = msg.to_term();
    let parsed = ControlMessage::from_term(&term).unwrap();

    match parsed {
        ControlMessage::AliasSendTt { .. } => {}
        _ => panic!("Expected AliasSendTt variant"),
    }
}

#[test]
fn test_alias_send_roundtrip() {
    let msg = ControlMessage::AliasSend {
        from_pid: make_pid(1, 0, 0),
        alias: make_reference(),
    };

    let term = msg.to_term();
    let decoded = ControlMessage::from_term(&term).unwrap();

    assert_eq!(msg, decoded);
}

//
// Unlink ID Tests
//

#[test]
fn test_unlink_id_roundtrip() {
    let msg = ControlMessage::UnlinkId {
        id: 42,
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
    };

    let term = msg.to_term();
    let decoded = ControlMessage::from_term(&term).unwrap();

    assert_eq!(msg, decoded);
}

#[test]
fn test_unlink_id_ack_roundtrip() {
    let msg = ControlMessage::UnlinkIdAck {
        id: 42,
        from_pid: make_pid(1, 0, 0),
        to_pid: make_pid(2, 0, 0),
    };

    let term = msg.to_term();
    let decoded = ControlMessage::from_term(&term).unwrap();

    assert_eq!(msg, decoded);
}

//
// Generic Message Tests
//

#[test]
fn test_generic_control_message() {
    let term = OwnedTerm::Tuple(vec![
        OwnedTerm::Integer(99),
        OwnedTerm::Atom(Atom::new("test")),
        OwnedTerm::Integer(42),
    ]);

    let msg = ControlMessage::from_term(&term).unwrap();

    match &msg {
        ControlMessage::Generic {
            message_type,
            fields,
        } => {
            assert_eq!(*message_type, 99);
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected Generic control message"),
    }

    let reconstructed = msg.to_term();
    assert_eq!(term, reconstructed);
}

//
// Edge Cases
//

#[test]
fn test_control_message_wrong_arity_send() {
    let term = OwnedTerm::tuple(vec![OwnedTerm::integer(2)]);

    let result = ControlMessage::from_term(&term);
    assert!(
        result.is_ok(),
        "Arity mismatch falls back to generic message"
    );
    assert!(matches!(result.unwrap(), ControlMessage::Generic { .. }));
}

#[test]
fn test_control_message_wrong_arity_link() {
    let term = OwnedTerm::tuple(vec![OwnedTerm::integer(1), OwnedTerm::integer(123)]);

    let result = ControlMessage::from_term(&term);
    assert!(
        result.is_ok(),
        "Arity mismatch falls back to generic message"
    );
    assert!(matches!(result.unwrap(), ControlMessage::Generic { .. }));
}

#[test]
fn test_control_message_empty_tuple() {
    let term = OwnedTerm::tuple(vec![]);

    let result = ControlMessage::from_term(&term);
    assert!(result.is_err(), "Empty tuple should fail");
}

#[test]
fn test_control_message_non_integer_type() {
    let term = OwnedTerm::tuple(vec![
        OwnedTerm::atom("not_a_number"),
        OwnedTerm::integer(1),
        OwnedTerm::integer(2),
    ]);

    let result = ControlMessage::from_term(&term);
    assert!(result.is_err(), "Non-integer message type should fail");
}

#[test]
fn test_control_message_type_out_of_range() {
    let term = OwnedTerm::tuple(vec![OwnedTerm::integer(300)]);

    let result = ControlMessage::from_term(&term);
    assert!(result.is_err(), "Message type > 255 should fail");
}

#[test]
fn test_control_message_negative_type() {
    let term = OwnedTerm::tuple(vec![OwnedTerm::integer(-1)]);

    let result = ControlMessage::from_term(&term);
    assert!(result.is_err(), "Negative message type should fail");
}

#[test]
fn test_control_message_not_tuple() {
    let term = OwnedTerm::list(vec![OwnedTerm::integer(2), OwnedTerm::integer(1)]);

    let result = ControlMessage::from_term(&term);
    assert!(result.is_err(), "Non-tuple should fail");
}

#[test]
fn test_control_message_unlink_id_negative() {
    let term = OwnedTerm::tuple(vec![
        OwnedTerm::integer(35),
        OwnedTerm::integer(-1),
        OwnedTerm::integer(0),
        OwnedTerm::integer(0),
    ]);

    let result = ControlMessage::from_term(&term);
    assert!(result.is_err(), "UNLINK_ID with negative id should fail");
}

#[test]
fn test_control_message_generic_unknown_type() {
    let term = OwnedTerm::tuple(vec![
        OwnedTerm::integer(99),
        OwnedTerm::integer(1),
        OwnedTerm::integer(2),
    ]);

    let result = ControlMessage::from_term(&term);
    assert!(
        result.is_ok(),
        "Unknown control message type should parse as Generic"
    );

    if let Ok(ControlMessage::Generic {
        message_type,
        fields,
    }) = result
    {
        assert_eq!(message_type, 99);
        assert_eq!(fields.len(), 2);
    } else {
        panic!("Should have parsed as Generic");
    }
}

#[test]
fn test_spawn_request_into_term_matches_to_term() {
    let msg = ControlMessage::SpawnRequest {
        req_id: make_reference(),
        from: make_pid(1, 0, 0),
        group_leader: make_pid(2, 0, 0),
        mfa: OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("erlang")),
            OwnedTerm::Atom(Atom::new("apply")),
            OwnedTerm::Integer(2),
        ]),
        arg_list: OwnedTerm::List(vec![OwnedTerm::Integer(42)]),
        opt_list: OwnedTerm::List(vec![OwnedTerm::Integer(99)]),
    };

    let via_to_term = msg.to_term();
    let via_into_term = msg.clone().into_term();

    assert_eq!(
        via_to_term, via_into_term,
        "to_term() and into_term() must produce identical output"
    );
}

#[test]
fn test_spawn_request_tt_into_term_matches_to_term() {
    let msg = ControlMessage::SpawnRequestTt {
        req_id: make_reference(),
        from: make_pid(1, 0, 0),
        group_leader: make_pid(2, 0, 0),
        mfa: OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("erlang")),
            OwnedTerm::Atom(Atom::new("apply")),
            OwnedTerm::Integer(2),
        ]),
        arg_list: OwnedTerm::List(vec![OwnedTerm::Integer(42)]),
        opt_list: OwnedTerm::List(vec![OwnedTerm::Integer(99)]),
        trace_token: OwnedTerm::Integer(123),
    };

    let via_to_term = msg.to_term();
    let via_into_term = msg.clone().into_term();

    assert_eq!(
        via_to_term, via_into_term,
        "to_term() and into_term() must produce identical output"
    );
}
