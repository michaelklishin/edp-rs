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
use crate::types::SequenceId;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::trace;

pub const DIST_FRAG_HEADER: u8 = 69;
pub const DIST_FRAG_CONT: u8 = 70;
pub const DEFAULT_FRAGMENT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_FRAGMENTS_VEC: u64 = 100_000;
const MAX_FRAGMENT_COUNT: u64 = 1_000_000;

#[derive(Debug, Clone)]
pub struct FragmentHeader {
    pub sequence_id: SequenceId,
    pub fragment_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FragmentCount(u64);

impl FragmentCount {
    pub fn new(count: u64) -> Result<Self> {
        if count == 0 {
            return Err(Error::InvalidStateMessage(
                "Fragment count cannot be zero".to_string(),
            ));
        }
        if count > MAX_FRAGMENT_COUNT {
            return Err(Error::InvalidStateMessage(format!(
                "Fragment count {} exceeds maximum of {}",
                count, MAX_FRAGMENT_COUNT
            )));
        }
        Ok(FragmentCount(count))
    }

    #[inline]
    pub fn get(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn exceeds_vec_limit(self) -> bool {
        self.0 > MAX_FRAGMENTS_VEC
    }
}

#[derive(Debug)]
struct FragmentedMessage {
    total_fragments: Option<FragmentCount>,
    fragments: Vec<Option<Vec<u8>>>,
    pending_fragments: HashMap<u64, Vec<u8>>,
    received_count: usize,
    atom_cache_data: Option<Vec<u8>>,
    last_update: Instant,
}

impl FragmentedMessage {
    fn new(
        _sequence_id: u64,
        total_fragments: Option<FragmentCount>,
        atom_cache_data: Option<Vec<u8>>,
    ) -> Self {
        let fragments = if let Some(count) = total_fragments {
            if count.exceeds_vec_limit() {
                trace!(
                    "Fragment count {} exceeds reasonable limit, will use pending map",
                    count.get()
                );
                Vec::new()
            } else {
                vec![None; count.get() as usize]
            }
        } else {
            Vec::new()
        };
        Self {
            total_fragments,
            fragments,
            pending_fragments: HashMap::new(),
            received_count: 0,
            atom_cache_data,
            last_update: Instant::now(),
        }
    }

    fn add_fragment(&mut self, fragment_id: u64, data: Vec<u8>) {
        self.last_update = Instant::now();

        if fragment_id == 0 {
            trace!("Ignoring fragment with id 0");
            return;
        }

        if let Some(count) = self.total_fragments {
            if fragment_id <= count.get() {
                let idx = (fragment_id - 1) as usize;
                if idx < self.fragments.len() {
                    if self.fragments[idx].is_some() {
                        trace!("Received duplicate fragment {} - ignoring", fragment_id);
                    } else {
                        self.fragments[idx] = Some(data);
                        self.received_count += 1;
                    }
                }
            }
        } else if let std::collections::hash_map::Entry::Vacant(e) =
            self.pending_fragments.entry(fragment_id)
        {
            e.insert(data);
        } else {
            trace!(
                "Received duplicate pending fragment {} - ignoring",
                fragment_id
            );
        }
    }

    fn set_total_fragments(&mut self, count: FragmentCount) {
        if self.total_fragments.as_ref() != Some(&count) {
            self.total_fragments = Some(count);

            if !count.exceeds_vec_limit() {
                self.fragments.resize(count.get() as usize, None);

                let pending: Vec<_> = self.pending_fragments.drain().collect();
                for (fragment_id, data) in pending {
                    if fragment_id > 0 && fragment_id <= count.get() {
                        let idx = (fragment_id - 1) as usize;
                        if idx < self.fragments.len() && self.fragments[idx].is_none() {
                            self.fragments[idx] = Some(data);
                            self.received_count += 1;
                        }
                    }
                }
            }
        }
    }

    fn is_complete(&self) -> bool {
        self.total_fragments
            .map(|count| self.received_count == count.get() as usize)
            .unwrap_or(false)
    }

    fn is_expired(&self, timeout: Duration) -> bool {
        self.last_update.elapsed() > timeout
    }

    fn reassemble(mut self) -> Option<Vec<u8>> {
        if !self.is_complete() {
            return None;
        }

        let cache_size = self.atom_cache_data.as_ref().map(|d| d.len()).unwrap_or(0);
        let fragments_size: usize = self
            .fragments
            .iter()
            .filter_map(|f| f.as_ref().map(|v| v.len()))
            .sum();
        let total_size = cache_size + fragments_size;

        let mut result = Vec::with_capacity(total_size);

        if let Some(cache_data) = self.atom_cache_data.take() {
            result.extend_from_slice(&cache_data);
        }

        for fragment in self.fragments.into_iter().flatten() {
            result.extend_from_slice(&fragment);
        }

        Some(result)
    }
}

#[derive(Debug)]
pub struct FragmentAssembler {
    pending: HashMap<SequenceId, FragmentedMessage>,
    fragment_timeout: Duration,
}

impl Default for FragmentAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl FragmentAssembler {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            fragment_timeout: DEFAULT_FRAGMENT_TIMEOUT,
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            pending: HashMap::new(),
            fragment_timeout: timeout,
        }
    }

    pub fn start_fragment<S: Into<SequenceId>>(
        &mut self,
        sequence_id: S,
        fragment_id: u64,
        atom_cache_data: Option<Vec<u8>>,
        payload: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let sequence_id = sequence_id.into();
        trace!(
            "Starting fragment sequence {}, fragment {} (counting down from {})",
            sequence_id.0, fragment_id, fragment_id
        );

        let count = match FragmentCount::new(fragment_id) {
            Ok(c) => c,
            Err(_) => {
                trace!(
                    "Invalid fragment count {} for sequence {}",
                    fragment_id, sequence_id.0
                );
                return None;
            }
        };

        if let Some(msg) = self.pending.get_mut(&sequence_id) {
            trace!(
                "Received header for sequence {} which already has buffered fragments",
                sequence_id.0
            );
            msg.set_total_fragments(count);
            msg.atom_cache_data = atom_cache_data;
            msg.add_fragment(fragment_id, payload);

            if msg.is_complete() {
                trace!("Fragment sequence {} now complete", sequence_id.0);
                if let Some(msg) = self.pending.remove(&sequence_id) {
                    return msg.reassemble();
                }
            }
            None
        } else {
            let mut msg = FragmentedMessage::new(sequence_id.0, Some(count), atom_cache_data);
            msg.add_fragment(fragment_id, payload);

            if msg.is_complete() {
                trace!("Fragment sequence {} complete immediately", sequence_id.0);
                msg.reassemble()
            } else {
                self.pending.insert(sequence_id, msg);
                None
            }
        }
    }

    pub fn add_fragment<S: Into<SequenceId>>(
        &mut self,
        sequence_id: S,
        fragment_id: u64,
        payload: Vec<u8>,
    ) -> Option<Vec<u8>> {
        let sequence_id = sequence_id.into();
        trace!(
            "Adding fragment {} to sequence {}",
            fragment_id, sequence_id.0
        );

        if let Some(msg) = self.pending.get_mut(&sequence_id) {
            msg.add_fragment(fragment_id, payload);

            if msg.is_complete() {
                trace!("Fragment sequence {} is now complete", sequence_id.0);
                if let Some(msg) = self.pending.remove(&sequence_id) {
                    return msg.reassemble();
                }
            }
        } else {
            trace!(
                "Received continuation fragment {} before header for sequence {} - buffering",
                fragment_id, sequence_id.0
            );
            let mut msg = FragmentedMessage::new(sequence_id.0, None, None);
            msg.add_fragment(fragment_id, payload);
            self.pending.insert(sequence_id, msg);
        }

        None
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let timeout = self.fragment_timeout;
        let before = self.pending.len();

        self.pending.retain(|seq_id, msg| {
            if msg.is_expired(timeout) {
                trace!("Dropping expired fragment sequence {}", seq_id.0);
                false
            } else {
                true
            }
        });

        before - self.pending.len()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}
