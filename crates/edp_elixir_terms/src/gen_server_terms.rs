// Copyright (C) 2025 Michael S. Klishin and Contributors
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

//! GenServer term construction and parsing.
//!
//! Helpers for constructing and parsing GenServer message tuples.
//! These are low-level building blocks, not a GenServer framework.

use erltf::{Atom, ExternalPid, OwnedTerm};

/// Helpers for constructing and parsing GenServer message tuples.
pub struct GenServerTerms;

impl GenServerTerms {
    /// Creates a `{:'$gen_call', {pid, ref}, request}` message.
    #[must_use]
    pub fn gen_call(from_pid: OwnedTerm, from_ref: OwnedTerm, request: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("$gen_call")),
            OwnedTerm::Tuple(vec![from_pid, from_ref]),
            request,
        ])
    }

    /// Creates a `{:'$gen_cast', request}` message.
    #[must_use]
    pub fn gen_cast(request: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("$gen_cast")), request])
    }

    /// Creates a `{:reply, reply, new_state}` tuple.
    #[must_use]
    pub fn reply(reply: OwnedTerm, new_state: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("reply")), reply, new_state])
    }

    /// Creates a `{:reply, reply, new_state, timeout}` tuple.
    #[must_use]
    pub fn reply_with_timeout(reply: OwnedTerm, new_state: OwnedTerm, timeout: i64) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("reply")),
            reply,
            new_state,
            OwnedTerm::Integer(timeout),
        ])
    }

    /// Creates a `{:reply, reply, new_state, :hibernate}` tuple.
    #[must_use]
    pub fn reply_hibernate(reply: OwnedTerm, new_state: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("reply")),
            reply,
            new_state,
            OwnedTerm::Atom(Atom::new("hibernate")),
        ])
    }

    /// Creates a `{:noreply, new_state}` tuple.
    #[must_use]
    pub fn noreply(new_state: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("noreply")), new_state])
    }

    /// Creates a `{:noreply, new_state, timeout}` tuple.
    #[must_use]
    pub fn noreply_with_timeout(new_state: OwnedTerm, timeout: i64) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("noreply")),
            new_state,
            OwnedTerm::Integer(timeout),
        ])
    }

    /// Creates a `{:noreply, new_state, :hibernate}` tuple.
    #[must_use]
    pub fn noreply_hibernate(new_state: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("noreply")),
            new_state,
            OwnedTerm::Atom(Atom::new("hibernate")),
        ])
    }

    /// Creates a `{:noreply, new_state, {:continue, arg}}` tuple.
    ///
    /// The continue callback will be invoked after the current callback returns.
    #[must_use]
    pub fn noreply_continue(new_state: OwnedTerm, continue_arg: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("noreply")),
            new_state,
            Self::continue_tuple(continue_arg),
        ])
    }

    /// Creates a `{:reply, reply, new_state, {:continue, arg}}` tuple.
    ///
    /// The continue callback will be invoked after the current callback returns.
    #[must_use]
    pub fn reply_continue(
        reply: OwnedTerm,
        new_state: OwnedTerm,
        continue_arg: OwnedTerm,
    ) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("reply")),
            reply,
            new_state,
            Self::continue_tuple(continue_arg),
        ])
    }

    /// Creates a `{:continue, arg}` tuple.
    #[must_use]
    pub fn continue_tuple(arg: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("continue")), arg])
    }

    /// Creates a `{:stop, reason, reply, new_state}` tuple.
    #[must_use]
    pub fn stop_with_reply(reason: OwnedTerm, reply: OwnedTerm, new_state: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("stop")),
            reason,
            reply,
            new_state,
        ])
    }

    /// Creates a `{:stop, reason, new_state}` tuple.
    #[must_use]
    pub fn stop(reason: OwnedTerm, new_state: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("stop")), reason, new_state])
    }

    /// Creates a `{:stop, :normal, state}` tuple.
    #[must_use]
    pub fn stop_normal(state: OwnedTerm) -> OwnedTerm {
        Self::stop(OwnedTerm::Atom(Atom::new("normal")), state)
    }

    /// Creates a `{:stop, :shutdown, state}` tuple.
    #[must_use]
    pub fn stop_shutdown(state: OwnedTerm) -> OwnedTerm {
        Self::stop(OwnedTerm::Atom(Atom::new("shutdown")), state)
    }

    /// Creates a `{:ok, state}` init response.
    #[must_use]
    pub fn init_ok(state: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("ok")), state])
    }

    /// Creates a `{:ok, state, timeout}` init response.
    #[must_use]
    pub fn init_ok_with_timeout(state: OwnedTerm, timeout: i64) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("ok")),
            state,
            OwnedTerm::Integer(timeout),
        ])
    }

    /// Creates a `{:ok, state, :hibernate}` init response.
    #[must_use]
    pub fn init_ok_hibernate(state: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("ok")),
            state,
            OwnedTerm::Atom(Atom::new("hibernate")),
        ])
    }

    /// Creates a `{:ok, state, {:continue, arg}}` init response.
    #[must_use]
    pub fn init_ok_continue(state: OwnedTerm, continue_arg: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("ok")),
            state,
            Self::continue_tuple(continue_arg),
        ])
    }

    /// Creates a `{:stop, reason}` init response.
    #[must_use]
    pub fn init_stop(reason: OwnedTerm) -> OwnedTerm {
        OwnedTerm::Tuple(vec![OwnedTerm::Atom(Atom::new("stop")), reason])
    }

    /// Creates an `:ignore` init response.
    #[must_use]
    pub fn init_ignore() -> OwnedTerm {
        OwnedTerm::Atom(Atom::new("ignore"))
    }

    /// Checks if the term is a `{:'$gen_call', from, request}` message.
    #[must_use]
    pub fn is_gen_call(term: &OwnedTerm) -> bool {
        matches!(
            term.as_3_tuple(),
            Some((first, _, _)) if first.is_atom_with_name("$gen_call")
        )
    }

    /// Checks if the term is a `{:'$gen_cast', request}` message.
    #[must_use]
    pub fn is_gen_cast(term: &OwnedTerm) -> bool {
        matches!(
            term.as_2_tuple(),
            Some((first, _)) if first.is_atom_with_name("$gen_cast")
        )
    }

    /// Extracts the from tuple and request from a gen_call message.
    #[must_use]
    pub fn parse_gen_call(term: &OwnedTerm) -> Option<(&OwnedTerm, &OwnedTerm)> {
        term.as_3_tuple().and_then(|(first, from, request)| {
            if first.is_atom_with_name("$gen_call") {
                Some((from, request))
            } else {
                None
            }
        })
    }

    /// Extracts the request from a gen_cast message.
    #[must_use]
    pub fn parse_gen_cast(term: &OwnedTerm) -> Option<&OwnedTerm> {
        term.as_2_tuple().and_then(|(first, request)| {
            if first.is_atom_with_name("$gen_cast") {
                Some(request)
            } else {
                None
            }
        })
    }

    /// Extracts the PID and reference from a gen_call "from" tuple.
    #[must_use]
    pub fn parse_from(from: &OwnedTerm) -> Option<(&ExternalPid, &OwnedTerm)> {
        from.as_2_tuple()
            .and_then(|(pid_term, ref_term)| pid_term.as_pid().map(|pid| (pid, ref_term)))
    }

    /// Checks if the term is a `{:reply, ...}` response.
    #[must_use]
    pub fn is_reply(term: &OwnedTerm) -> bool {
        term.as_tuple()
            .is_some_and(|t| t.first().is_some_and(|f| f.is_atom_with_name("reply")))
    }

    /// Checks if the term is a `{:noreply, ...}` response.
    #[must_use]
    pub fn is_noreply(term: &OwnedTerm) -> bool {
        term.as_tuple()
            .is_some_and(|t| t.first().is_some_and(|f| f.is_atom_with_name("noreply")))
    }

    /// Checks if the term is a `{:stop, ...}` response.
    #[must_use]
    pub fn is_stop(term: &OwnedTerm) -> bool {
        term.as_tuple()
            .is_some_and(|t| t.first().is_some_and(|f| f.is_atom_with_name("stop")))
    }

    /// Checks if the term is a `{:continue, arg}` tuple.
    #[must_use]
    pub fn is_continue(term: &OwnedTerm) -> bool {
        matches!(
            term.as_2_tuple(),
            Some((first, _)) if first.is_atom_with_name("continue")
        )
    }

    /// Extracts the argument from a `{:continue, arg}` tuple.
    #[must_use]
    pub fn parse_continue(term: &OwnedTerm) -> Option<&OwnedTerm> {
        term.as_2_tuple().and_then(|(first, arg)| {
            if first.is_atom_with_name("continue") {
                Some(arg)
            } else {
                None
            }
        })
    }

    /// Checks if a response tuple has a continue action as its last element.
    #[must_use]
    pub fn has_continue(term: &OwnedTerm) -> bool {
        term.as_tuple()
            .is_some_and(|t| t.last().is_some_and(Self::is_continue))
    }
}
