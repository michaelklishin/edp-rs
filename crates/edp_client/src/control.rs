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

//! Distribution protocol control messages.
//!
//! Control messages are sent between connected nodes for process management,
//! monitoring, linking, and message passing.

use crate::errors::{Error, Result};
use erltf::OwnedTerm;
use std::convert::TryFrom;
use std::mem;

/// Control message types (first element of control tuple)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ControlMessageType {
    Link = 1,
    Send = 2,
    Exit = 3,
    Unlink = 4,
    NodeLink = 5,
    RegSend = 6,
    GroupLeader = 7,
    Exit2 = 8,
    SendTt = 12,
    ExitTt = 13,
    RegSendTt = 16,
    Exit2Tt = 18,
    MonitorP = 19,
    DemonitorP = 20,
    MonitorPExit = 21,
    SendSender = 22,
    SendSenderTt = 23,
    PayloadExit = 24,
    PayloadExitTt = 25,
    PayloadExit2 = 26,
    PayloadExit2Tt = 27,
    PayloadMonitorPExit = 28,
    SpawnRequest = 29,
    SpawnRequestTt = 30,
    SpawnReply = 31,
    SpawnReplyTt = 32,
    UnlinkId = 35,
    UnlinkIdAck = 36,
    AliasSend = 33,
    AliasSendTt = 38,
}

impl TryFrom<u8> for ControlMessageType {
    type Error = u8;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Link),
            2 => Ok(Self::Send),
            3 => Ok(Self::Exit),
            4 => Ok(Self::Unlink),
            5 => Ok(Self::NodeLink),
            6 => Ok(Self::RegSend),
            7 => Ok(Self::GroupLeader),
            8 => Ok(Self::Exit2),
            12 => Ok(Self::SendTt),
            13 => Ok(Self::ExitTt),
            16 => Ok(Self::RegSendTt),
            18 => Ok(Self::Exit2Tt),
            19 => Ok(Self::MonitorP),
            20 => Ok(Self::DemonitorP),
            21 => Ok(Self::MonitorPExit),
            22 => Ok(Self::SendSender),
            23 => Ok(Self::SendSenderTt),
            24 => Ok(Self::PayloadExit),
            25 => Ok(Self::PayloadExitTt),
            26 => Ok(Self::PayloadExit2),
            27 => Ok(Self::PayloadExit2Tt),
            28 => Ok(Self::PayloadMonitorPExit),
            29 => Ok(Self::SpawnRequest),
            30 => Ok(Self::SpawnRequestTt),
            31 => Ok(Self::SpawnReply),
            32 => Ok(Self::SpawnReplyTt),
            35 => Ok(Self::UnlinkId),
            36 => Ok(Self::UnlinkIdAck),
            33 => Ok(Self::AliasSend),
            38 => Ok(Self::AliasSendTt),
            _ => Err(value),
        }
    }
}

impl ControlMessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        value.try_into().ok()
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Control message representation
#[derive(Debug, Clone, PartialEq)]
pub enum ControlMessage {
    /// LINK {1, FromPid, ToPid}
    Link {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// SEND {2, Cookie, ToPid}
    Send {
        cookie: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// EXIT {3, FromPid, ToPid, Reason}
    Exit {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
        reason: OwnedTerm,
    },

    /// UNLINK_ID {35, Id, FromPid, ToPid}
    UnlinkId {
        id: u64,
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// UNLINK_ID_ACK {36, Id, FromPid, ToPid}
    UnlinkIdAck {
        id: u64,
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// REG_SEND {6, FromPid, Cookie, ToName}
    RegSend {
        from_pid: OwnedTerm,
        cookie: OwnedTerm,
        to_name: OwnedTerm,
    },

    /// MONITOR_P {19, FromPid, ToProc, Ref}
    MonitorP {
        from_pid: OwnedTerm,
        to_proc: OwnedTerm,
        reference: OwnedTerm,
    },

    /// DEMONITOR_P {20, FromPid, ToProc, Ref}
    DemonitorP {
        from_pid: OwnedTerm,
        to_proc: OwnedTerm,
        reference: OwnedTerm,
    },

    /// MONITOR_P_EXIT {21, FromProc, ToPid, Ref, Reason}
    MonitorPExit {
        from_proc: OwnedTerm,
        to_pid: OwnedTerm,
        reference: OwnedTerm,
        reason: OwnedTerm,
    },

    /// SPAWN_REQUEST {29, ReqId, From, GroupLeader, {Module, Function, Arity}, ArgList, OptList}
    SpawnRequest {
        req_id: OwnedTerm,
        from: OwnedTerm,
        group_leader: OwnedTerm,
        mfa: OwnedTerm,
        arg_list: OwnedTerm,
        opt_list: OwnedTerm,
    },

    /// SPAWN_REPLY {31, ReqId, To, Flags, Result}
    SpawnReply {
        req_id: OwnedTerm,
        to: OwnedTerm,
        flags: OwnedTerm,
        result: OwnedTerm,
    },

    /// ALIAS_SEND {33, FromPid, Alias}
    AliasSend {
        from_pid: OwnedTerm,
        alias: OwnedTerm,
    },

    /// UNLINK {4, FromPid, ToPid} (deprecated, use UnlinkId instead)
    Unlink {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// NODE_LINK {5}
    NodeLink,

    /// GROUP_LEADER {7, FromPid, ToPid}
    GroupLeader {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// EXIT2 {8, FromPid, ToPid, Reason}
    Exit2 {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
        reason: OwnedTerm,
    },

    /// SEND_SENDER {22, FromPid, ToPid}
    SendSender {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// PAYLOAD_EXIT {24, FromPid, ToPid}
    PayloadExit {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// PAYLOAD_EXIT2 {26, FromPid, ToPid}
    PayloadExit2 {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
    },

    /// PAYLOAD_MONITOR_P_EXIT {28, FromProc, ToPid, Ref}
    PayloadMonitorPExit {
        from_proc: OwnedTerm,
        to_pid: OwnedTerm,
        reference: OwnedTerm,
    },

    /// SEND_TT {12, Cookie, ToPid, TraceToken}
    SendTt {
        cookie: OwnedTerm,
        to_pid: OwnedTerm,
        trace_token: OwnedTerm,
    },

    /// EXIT_TT {13, FromPid, ToPid, TraceToken, Reason}
    ExitTt {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
        trace_token: OwnedTerm,
        reason: OwnedTerm,
    },

    /// REG_SEND_TT {16, FromPid, Cookie, ToName, TraceToken}
    RegSendTt {
        from_pid: OwnedTerm,
        cookie: OwnedTerm,
        to_name: OwnedTerm,
        trace_token: OwnedTerm,
    },

    /// EXIT2_TT {18, FromPid, ToPid, TraceToken, Reason}
    Exit2Tt {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
        trace_token: OwnedTerm,
        reason: OwnedTerm,
    },

    /// SEND_SENDER_TT {23, FromPid, ToPid, TraceToken}
    SendSenderTt {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
        trace_token: OwnedTerm,
    },

    /// PAYLOAD_EXIT_TT {25, FromPid, ToPid, TraceToken}
    PayloadExitTt {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
        trace_token: OwnedTerm,
    },

    /// PAYLOAD_EXIT2_TT {27, FromPid, ToPid, TraceToken}
    PayloadExit2Tt {
        from_pid: OwnedTerm,
        to_pid: OwnedTerm,
        trace_token: OwnedTerm,
    },

    /// SPAWN_REQUEST_TT {30, ReqId, From, GroupLeader, {Module, Function, Arity}, ArgList, OptList, TraceToken}
    SpawnRequestTt {
        req_id: OwnedTerm,
        from: OwnedTerm,
        group_leader: OwnedTerm,
        mfa: OwnedTerm,
        arg_list: OwnedTerm,
        opt_list: OwnedTerm,
        trace_token: OwnedTerm,
    },

    /// SPAWN_REPLY_TT {32, ReqId, To, Flags, Result, TraceToken}
    SpawnReplyTt {
        req_id: OwnedTerm,
        to: OwnedTerm,
        flags: OwnedTerm,
        result: OwnedTerm,
        trace_token: OwnedTerm,
    },

    /// ALIAS_SEND_TT {38, FromPid, Alias, TraceToken}
    AliasSendTt {
        from_pid: OwnedTerm,
        alias: OwnedTerm,
        trace_token: OwnedTerm,
    },

    /// Generic control message (for unsupported types)
    Generic {
        message_type: u8,
        fields: Vec<OwnedTerm>,
    },
}

impl ControlMessage {
    /// Parse a control message from an Erlang term (tuple)
    pub fn from_term(term: &OwnedTerm) -> Result<Self> {
        let mut elements = term
            .as_tuple()
            .ok_or_else(|| {
                Error::InvalidControlMessage("Control message must be a tuple".to_string())
            })?
            .to_vec();

        if elements.is_empty() {
            return Err(Error::InvalidControlMessage(
                "Control message tuple is empty".to_string(),
            ));
        }

        let msg_type_raw = elements[0].as_integer().ok_or_else(|| {
            Error::InvalidControlMessage("Message type must be an integer".to_string())
        })?;

        if !(0..=255).contains(&msg_type_raw) {
            return Err(Error::InvalidControlMessage(format!(
                "Message type out of range: {}",
                msg_type_raw
            )));
        }
        let msg_type = msg_type_raw as u8;

        match ControlMessageType::from_u8(msg_type) {
            Some(ControlMessageType::Link) if elements.len() == 3 => Ok(ControlMessage::Link {
                from_pid: mem::take(&mut elements[1]),
                to_pid: mem::take(&mut elements[2]),
            }),

            Some(ControlMessageType::Send) if elements.len() == 3 => Ok(ControlMessage::Send {
                cookie: mem::take(&mut elements[1]),
                to_pid: mem::take(&mut elements[2]),
            }),

            Some(ControlMessageType::Exit) if elements.len() == 4 => Ok(ControlMessage::Exit {
                from_pid: mem::take(&mut elements[1]),
                to_pid: mem::take(&mut elements[2]),
                reason: mem::take(&mut elements[3]),
            }),

            Some(ControlMessageType::UnlinkId) if elements.len() == 4 => {
                let id_raw = elements[1].as_integer().ok_or_else(|| {
                    Error::InvalidControlMessage("UNLINK_ID id must be an integer".to_string())
                })?;

                if id_raw < 0 {
                    return Err(Error::InvalidControlMessage(format!(
                        "UNLINK_ID id must be non-negative: {}",
                        id_raw
                    )));
                }

                Ok(ControlMessage::UnlinkId {
                    id: id_raw as u64,
                    from_pid: elements[2].clone(),
                    to_pid: elements[3].clone(),
                })
            }

            Some(ControlMessageType::UnlinkIdAck) if elements.len() == 4 => {
                let id_raw = elements[1].as_integer().ok_or_else(|| {
                    Error::InvalidControlMessage("UNLINK_ID_ACK id must be an integer".to_string())
                })?;

                if id_raw < 0 {
                    return Err(Error::InvalidControlMessage(format!(
                        "UNLINK_ID_ACK id must be non-negative: {}",
                        id_raw
                    )));
                }

                Ok(ControlMessage::UnlinkIdAck {
                    id: id_raw as u64,
                    from_pid: elements[2].clone(),
                    to_pid: elements[3].clone(),
                })
            }

            Some(ControlMessageType::RegSend) if elements.len() == 4 => {
                Ok(ControlMessage::RegSend {
                    from_pid: elements[1].clone(),
                    cookie: elements[2].clone(),
                    to_name: elements[3].clone(),
                })
            }

            Some(ControlMessageType::MonitorP) if elements.len() == 4 => {
                Ok(ControlMessage::MonitorP {
                    from_pid: elements[1].clone(),
                    to_proc: elements[2].clone(),
                    reference: elements[3].clone(),
                })
            }

            Some(ControlMessageType::DemonitorP) if elements.len() == 4 => {
                Ok(ControlMessage::DemonitorP {
                    from_pid: elements[1].clone(),
                    to_proc: elements[2].clone(),
                    reference: elements[3].clone(),
                })
            }

            Some(ControlMessageType::MonitorPExit) if elements.len() == 5 => {
                Ok(ControlMessage::MonitorPExit {
                    from_proc: elements[1].clone(),
                    to_pid: elements[2].clone(),
                    reference: elements[3].clone(),
                    reason: elements[4].clone(),
                })
            }

            Some(ControlMessageType::SpawnRequest) if elements.len() == 7 => {
                Ok(ControlMessage::SpawnRequest {
                    req_id: elements[1].clone(),
                    from: elements[2].clone(),
                    group_leader: elements[3].clone(),
                    mfa: elements[4].clone(),
                    arg_list: elements[5].clone(),
                    opt_list: elements[6].clone(),
                })
            }

            Some(ControlMessageType::SpawnReply) if elements.len() == 5 => {
                Ok(ControlMessage::SpawnReply {
                    req_id: elements[1].clone(),
                    to: elements[2].clone(),
                    flags: elements[3].clone(),
                    result: elements[4].clone(),
                })
            }

            Some(ControlMessageType::AliasSend) if elements.len() == 3 => {
                Ok(ControlMessage::AliasSend {
                    from_pid: elements[1].clone(),
                    alias: elements[2].clone(),
                })
            }

            Some(ControlMessageType::Unlink) if elements.len() == 3 => Ok(ControlMessage::Unlink {
                from_pid: elements[1].clone(),
                to_pid: elements[2].clone(),
            }),

            Some(ControlMessageType::NodeLink) if elements.len() == 1 => {
                Ok(ControlMessage::NodeLink)
            }

            Some(ControlMessageType::GroupLeader) if elements.len() == 3 => {
                Ok(ControlMessage::GroupLeader {
                    from_pid: elements[1].clone(),
                    to_pid: elements[2].clone(),
                })
            }

            Some(ControlMessageType::Exit2) if elements.len() == 4 => Ok(ControlMessage::Exit2 {
                from_pid: elements[1].clone(),
                to_pid: elements[2].clone(),
                reason: elements[3].clone(),
            }),

            Some(ControlMessageType::SendSender) if elements.len() == 3 => {
                Ok(ControlMessage::SendSender {
                    from_pid: elements[1].clone(),
                    to_pid: elements[2].clone(),
                })
            }

            Some(ControlMessageType::PayloadExit) if elements.len() == 3 => {
                Ok(ControlMessage::PayloadExit {
                    from_pid: elements[1].clone(),
                    to_pid: elements[2].clone(),
                })
            }

            Some(ControlMessageType::PayloadExit2) if elements.len() == 3 => {
                Ok(ControlMessage::PayloadExit2 {
                    from_pid: elements[1].clone(),
                    to_pid: elements[2].clone(),
                })
            }

            Some(ControlMessageType::PayloadMonitorPExit) if elements.len() == 4 => {
                Ok(ControlMessage::PayloadMonitorPExit {
                    from_proc: elements[1].clone(),
                    to_pid: elements[2].clone(),
                    reference: elements[3].clone(),
                })
            }

            Some(ControlMessageType::SendTt) if elements.len() == 4 => Ok(ControlMessage::SendTt {
                cookie: elements[1].clone(),
                to_pid: elements[2].clone(),
                trace_token: elements[3].clone(),
            }),

            Some(ControlMessageType::ExitTt) if elements.len() == 5 => Ok(ControlMessage::ExitTt {
                from_pid: elements[1].clone(),
                to_pid: elements[2].clone(),
                trace_token: elements[3].clone(),
                reason: elements[4].clone(),
            }),

            Some(ControlMessageType::RegSendTt) if elements.len() == 5 => {
                Ok(ControlMessage::RegSendTt {
                    from_pid: elements[1].clone(),
                    cookie: elements[2].clone(),
                    to_name: elements[3].clone(),
                    trace_token: elements[4].clone(),
                })
            }

            Some(ControlMessageType::Exit2Tt) if elements.len() == 5 => {
                Ok(ControlMessage::Exit2Tt {
                    from_pid: elements[1].clone(),
                    to_pid: elements[2].clone(),
                    trace_token: elements[3].clone(),
                    reason: elements[4].clone(),
                })
            }

            Some(ControlMessageType::SendSenderTt) if elements.len() == 4 => {
                Ok(ControlMessage::SendSenderTt {
                    from_pid: elements[1].clone(),
                    to_pid: elements[2].clone(),
                    trace_token: elements[3].clone(),
                })
            }

            Some(ControlMessageType::PayloadExitTt) if elements.len() == 4 => {
                Ok(ControlMessage::PayloadExitTt {
                    from_pid: elements[1].clone(),
                    to_pid: elements[2].clone(),
                    trace_token: elements[3].clone(),
                })
            }

            Some(ControlMessageType::PayloadExit2Tt) if elements.len() == 4 => {
                Ok(ControlMessage::PayloadExit2Tt {
                    from_pid: elements[1].clone(),
                    to_pid: elements[2].clone(),
                    trace_token: elements[3].clone(),
                })
            }

            Some(ControlMessageType::SpawnRequestTt) if elements.len() == 8 => {
                Ok(ControlMessage::SpawnRequestTt {
                    req_id: elements[1].clone(),
                    from: elements[2].clone(),
                    group_leader: elements[3].clone(),
                    mfa: elements[4].clone(),
                    arg_list: elements[5].clone(),
                    opt_list: elements[6].clone(),
                    trace_token: elements[7].clone(),
                })
            }

            Some(ControlMessageType::SpawnReplyTt) if elements.len() == 6 => {
                Ok(ControlMessage::SpawnReplyTt {
                    req_id: elements[1].clone(),
                    to: elements[2].clone(),
                    flags: elements[3].clone(),
                    result: elements[4].clone(),
                    trace_token: elements[5].clone(),
                })
            }

            Some(ControlMessageType::AliasSendTt) if elements.len() == 4 => {
                Ok(ControlMessage::AliasSendTt {
                    from_pid: elements[1].clone(),
                    alias: elements[2].clone(),
                    trace_token: elements[3].clone(),
                })
            }

            _ => Ok(ControlMessage::Generic {
                message_type: msg_type,
                fields: elements[1..].to_vec(),
            }),
        }
    }

    /// Convert this control message to an Erlang term (tuple)
    pub fn to_term(&self) -> OwnedTerm {
        match self {
            ControlMessage::Link { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Link as i64),
                from_pid.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::Send { cookie, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Send as i64),
                cookie.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::Exit {
                from_pid,
                to_pid,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Exit as i64),
                from_pid.clone(),
                to_pid.clone(),
                reason.clone(),
            ]),

            ControlMessage::UnlinkId {
                id,
                from_pid,
                to_pid,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::UnlinkId as i64),
                OwnedTerm::Integer(*id as i64),
                from_pid.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::UnlinkIdAck {
                id,
                from_pid,
                to_pid,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::UnlinkIdAck as i64),
                OwnedTerm::Integer(*id as i64),
                from_pid.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::RegSend {
                from_pid,
                cookie,
                to_name,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::RegSend as i64),
                from_pid.clone(),
                cookie.clone(),
                to_name.clone(),
            ]),

            ControlMessage::MonitorP {
                from_pid,
                to_proc,
                reference,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::MonitorP as i64),
                from_pid.clone(),
                to_proc.clone(),
                reference.clone(),
            ]),

            ControlMessage::DemonitorP {
                from_pid,
                to_proc,
                reference,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::DemonitorP as i64),
                from_pid.clone(),
                to_proc.clone(),
                reference.clone(),
            ]),

            ControlMessage::MonitorPExit {
                from_proc,
                to_pid,
                reference,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::MonitorPExit as i64),
                from_proc.clone(),
                to_pid.clone(),
                reference.clone(),
                reason.clone(),
            ]),

            ControlMessage::SpawnRequest {
                req_id,
                from,
                group_leader,
                mfa,
                arg_list,
                opt_list,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SpawnRequest as i64),
                req_id.clone(),
                from.clone(),
                group_leader.clone(),
                mfa.clone(),
                arg_list.clone(),
                opt_list.clone(),
            ]),

            ControlMessage::SpawnReply {
                req_id,
                to,
                flags,
                result,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SpawnReply as i64),
                req_id.clone(),
                to.clone(),
                flags.clone(),
                result.clone(),
            ]),

            ControlMessage::AliasSend { from_pid, alias } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::AliasSend as i64),
                from_pid.clone(),
                alias.clone(),
            ]),

            ControlMessage::Unlink { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Unlink as i64),
                from_pid.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::NodeLink => OwnedTerm::Tuple(vec![OwnedTerm::Integer(
                ControlMessageType::NodeLink as i64,
            )]),

            ControlMessage::GroupLeader { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::GroupLeader as i64),
                from_pid.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::Exit2 {
                from_pid,
                to_pid,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Exit2 as i64),
                from_pid.clone(),
                to_pid.clone(),
                reason.clone(),
            ]),

            ControlMessage::SendSender { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SendSender as i64),
                from_pid.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::PayloadExit { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadExit as i64),
                from_pid.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::PayloadExit2 { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadExit2 as i64),
                from_pid.clone(),
                to_pid.clone(),
            ]),

            ControlMessage::PayloadMonitorPExit {
                from_proc,
                to_pid,
                reference,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadMonitorPExit as i64),
                from_proc.clone(),
                to_pid.clone(),
                reference.clone(),
            ]),

            ControlMessage::SendTt {
                cookie,
                to_pid,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SendTt as i64),
                cookie.clone(),
                to_pid.clone(),
                trace_token.clone(),
            ]),

            ControlMessage::ExitTt {
                from_pid,
                to_pid,
                trace_token,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::ExitTt as i64),
                from_pid.clone(),
                to_pid.clone(),
                trace_token.clone(),
                reason.clone(),
            ]),

            ControlMessage::RegSendTt {
                from_pid,
                cookie,
                to_name,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::RegSendTt as i64),
                from_pid.clone(),
                cookie.clone(),
                to_name.clone(),
                trace_token.clone(),
            ]),

            ControlMessage::Exit2Tt {
                from_pid,
                to_pid,
                trace_token,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Exit2Tt as i64),
                from_pid.clone(),
                to_pid.clone(),
                trace_token.clone(),
                reason.clone(),
            ]),

            ControlMessage::SendSenderTt {
                from_pid,
                to_pid,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SendSenderTt as i64),
                from_pid.clone(),
                to_pid.clone(),
                trace_token.clone(),
            ]),

            ControlMessage::PayloadExitTt {
                from_pid,
                to_pid,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadExitTt as i64),
                from_pid.clone(),
                to_pid.clone(),
                trace_token.clone(),
            ]),

            ControlMessage::PayloadExit2Tt {
                from_pid,
                to_pid,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadExit2Tt as i64),
                from_pid.clone(),
                to_pid.clone(),
                trace_token.clone(),
            ]),

            ControlMessage::SpawnRequestTt {
                req_id,
                from,
                group_leader,
                mfa,
                arg_list,
                opt_list,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SpawnRequestTt as i64),
                req_id.clone(),
                from.clone(),
                group_leader.clone(),
                mfa.clone(),
                arg_list.clone(),
                opt_list.clone(),
                trace_token.clone(),
            ]),

            ControlMessage::SpawnReplyTt {
                req_id,
                to,
                flags,
                result,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SpawnReplyTt as i64),
                req_id.clone(),
                to.clone(),
                flags.clone(),
                result.clone(),
                trace_token.clone(),
            ]),

            ControlMessage::AliasSendTt {
                from_pid,
                alias,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::AliasSendTt as i64),
                from_pid.clone(),
                alias.clone(),
                trace_token.clone(),
            ]),

            ControlMessage::Generic {
                message_type,
                fields,
            } => {
                let mut elements = vec![OwnedTerm::Integer(*message_type as i64)];
                elements.extend_from_slice(fields);
                OwnedTerm::Tuple(elements)
            }
        }
    }

    /// Convert this control message to an Erlang term (tuple), consuming self to avoid clones
    pub fn into_term(self) -> OwnedTerm {
        match self {
            ControlMessage::Link { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Link as i64),
                from_pid,
                to_pid,
            ]),

            ControlMessage::Send { cookie, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Send as i64),
                cookie,
                to_pid,
            ]),

            ControlMessage::Exit {
                from_pid,
                to_pid,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Exit as i64),
                from_pid,
                to_pid,
                reason,
            ]),

            ControlMessage::UnlinkId {
                id,
                from_pid,
                to_pid,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::UnlinkId as i64),
                OwnedTerm::Integer(id as i64),
                from_pid,
                to_pid,
            ]),

            ControlMessage::UnlinkIdAck {
                id,
                from_pid,
                to_pid,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::UnlinkIdAck as i64),
                OwnedTerm::Integer(id as i64),
                from_pid,
                to_pid,
            ]),

            ControlMessage::RegSend {
                from_pid,
                cookie,
                to_name,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::RegSend as i64),
                from_pid,
                cookie,
                to_name,
            ]),

            ControlMessage::MonitorP {
                from_pid,
                to_proc,
                reference,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::MonitorP as i64),
                from_pid,
                to_proc,
                reference,
            ]),

            ControlMessage::DemonitorP {
                from_pid,
                to_proc,
                reference,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::DemonitorP as i64),
                from_pid,
                to_proc,
                reference,
            ]),

            ControlMessage::MonitorPExit {
                from_proc,
                to_pid,
                reference,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::MonitorPExit as i64),
                from_proc,
                to_pid,
                reference,
                reason,
            ]),

            ControlMessage::SpawnRequest {
                req_id,
                from,
                group_leader,
                mfa,
                arg_list,
                opt_list,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SpawnRequest as i64),
                req_id,
                from,
                group_leader,
                mfa,
                arg_list,
                opt_list,
            ]),

            ControlMessage::SpawnRequestTt {
                req_id,
                from,
                group_leader,
                mfa,
                arg_list,
                opt_list,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SpawnRequestTt as i64),
                req_id,
                from,
                group_leader,
                mfa,
                arg_list,
                opt_list,
                trace_token,
            ]),

            ControlMessage::SpawnReply {
                req_id,
                to,
                flags,
                result,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SpawnReply as i64),
                req_id,
                to,
                flags,
                result,
            ]),

            ControlMessage::SpawnReplyTt {
                req_id,
                to,
                flags,
                result,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SpawnReplyTt as i64),
                req_id,
                to,
                flags,
                result,
                trace_token,
            ]),

            ControlMessage::AliasSend { from_pid, alias } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::AliasSend as i64),
                from_pid,
                alias,
            ]),

            ControlMessage::AliasSendTt {
                from_pid,
                alias,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::AliasSendTt as i64),
                from_pid,
                alias,
                trace_token,
            ]),

            ControlMessage::Unlink { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Unlink as i64),
                from_pid,
                to_pid,
            ]),

            ControlMessage::NodeLink => OwnedTerm::Tuple(vec![OwnedTerm::Integer(
                ControlMessageType::NodeLink as i64,
            )]),

            ControlMessage::Exit2 {
                from_pid,
                to_pid,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Exit2 as i64),
                from_pid,
                to_pid,
                reason,
            ]),

            ControlMessage::GroupLeader { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::GroupLeader as i64),
                from_pid,
                to_pid,
            ]),

            ControlMessage::SendSender { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SendSender as i64),
                from_pid,
                to_pid,
            ]),

            ControlMessage::PayloadExit { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadExit as i64),
                from_pid,
                to_pid,
            ]),

            ControlMessage::PayloadExit2 { from_pid, to_pid } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadExit2 as i64),
                from_pid,
                to_pid,
            ]),

            ControlMessage::PayloadMonitorPExit {
                from_proc,
                to_pid,
                reference,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadMonitorPExit as i64),
                from_proc,
                to_pid,
                reference,
            ]),

            ControlMessage::SendTt {
                cookie,
                to_pid,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SendTt as i64),
                cookie,
                to_pid,
                trace_token,
            ]),

            ControlMessage::ExitTt {
                from_pid,
                to_pid,
                trace_token,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::ExitTt as i64),
                from_pid,
                to_pid,
                trace_token,
                reason,
            ]),

            ControlMessage::RegSendTt {
                from_pid,
                cookie,
                to_name,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::RegSendTt as i64),
                from_pid,
                cookie,
                to_name,
                trace_token,
            ]),

            ControlMessage::Exit2Tt {
                from_pid,
                to_pid,
                trace_token,
                reason,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::Exit2Tt as i64),
                from_pid,
                to_pid,
                trace_token,
                reason,
            ]),

            ControlMessage::SendSenderTt {
                from_pid,
                to_pid,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::SendSenderTt as i64),
                from_pid,
                to_pid,
                trace_token,
            ]),

            ControlMessage::PayloadExitTt {
                from_pid,
                to_pid,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadExitTt as i64),
                from_pid,
                to_pid,
                trace_token,
            ]),

            ControlMessage::PayloadExit2Tt {
                from_pid,
                to_pid,
                trace_token,
            } => OwnedTerm::Tuple(vec![
                OwnedTerm::Integer(ControlMessageType::PayloadExit2Tt as i64),
                from_pid,
                to_pid,
                trace_token,
            ]),

            ControlMessage::Generic {
                message_type,
                fields,
            } => {
                let mut elements = vec![OwnedTerm::Integer(message_type as i64)];
                elements.extend(fields);
                OwnedTerm::Tuple(elements)
            }
        }
    }

    pub fn link(from_pid: OwnedTerm, to_pid: OwnedTerm) -> Self {
        ControlMessage::Link { from_pid, to_pid }
    }

    pub fn unlink(from_pid: OwnedTerm, to_pid: OwnedTerm) -> Self {
        ControlMessage::Unlink { from_pid, to_pid }
    }

    pub fn send(cookie: OwnedTerm, to_pid: OwnedTerm) -> Self {
        ControlMessage::Send { cookie, to_pid }
    }

    pub fn exit(from_pid: OwnedTerm, to_pid: OwnedTerm, reason: OwnedTerm) -> Self {
        ControlMessage::Exit {
            from_pid,
            to_pid,
            reason,
        }
    }

    pub fn exit2(from_pid: OwnedTerm, to_pid: OwnedTerm, reason: OwnedTerm) -> Self {
        ControlMessage::Exit2 {
            from_pid,
            to_pid,
            reason,
        }
    }

    pub fn reg_send(from_pid: OwnedTerm, cookie: OwnedTerm, to_name: OwnedTerm) -> Self {
        ControlMessage::RegSend {
            from_pid,
            cookie,
            to_name,
        }
    }

    pub fn group_leader(from_pid: OwnedTerm, to_pid: OwnedTerm) -> Self {
        ControlMessage::GroupLeader { from_pid, to_pid }
    }

    pub fn send_sender(from_pid: OwnedTerm, to_pid: OwnedTerm) -> Self {
        ControlMessage::SendSender { from_pid, to_pid }
    }

    pub fn monitor_p(from_pid: OwnedTerm, to_proc: OwnedTerm, reference: OwnedTerm) -> Self {
        ControlMessage::MonitorP {
            from_pid,
            to_proc,
            reference,
        }
    }

    pub fn demonitor_p(from_pid: OwnedTerm, to_proc: OwnedTerm, reference: OwnedTerm) -> Self {
        ControlMessage::DemonitorP {
            from_pid,
            to_proc,
            reference,
        }
    }

    pub fn monitor_p_exit(
        from_proc: OwnedTerm,
        to_pid: OwnedTerm,
        reference: OwnedTerm,
        reason: OwnedTerm,
    ) -> Self {
        ControlMessage::MonitorPExit {
            from_proc,
            to_pid,
            reference,
            reason,
        }
    }

    pub fn payload_exit(from_pid: OwnedTerm, to_pid: OwnedTerm) -> Self {
        ControlMessage::PayloadExit { from_pid, to_pid }
    }

    pub fn payload_exit2(from_pid: OwnedTerm, to_pid: OwnedTerm) -> Self {
        ControlMessage::PayloadExit2 { from_pid, to_pid }
    }

    pub fn payload_monitor_p_exit(
        from_proc: OwnedTerm,
        to_pid: OwnedTerm,
        reference: OwnedTerm,
    ) -> Self {
        ControlMessage::PayloadMonitorPExit {
            from_proc,
            to_pid,
            reference,
        }
    }
}
