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

//! MD5 digest computation for distribution protocol handshake.

use md5::{Digest, Md5};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::trace;

pub fn compute_digest(challenge: u32, cookie: &str) -> [u8; 16] {
    let challenge_str = challenge.to_string();
    let input = format!("{}{}", cookie, challenge_str);
    trace!(
        "Computing digest for: cookie='{}', challenge={}, input='{}'",
        cookie, challenge, input
    );

    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    trace!("Digest result: {:02x?}", result);
    result.into()
}

pub fn generate_challenge() -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();

    (nanos & 0xFFFF_FFFF) as u32
}
