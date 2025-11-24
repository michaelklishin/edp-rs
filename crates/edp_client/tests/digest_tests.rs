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

use edp_client::digest::{compute_digest, generate_challenge};
use std::thread;
use std::time::Duration;

//
// Digest Computation
//

#[test]
fn test_compute_digest() {
    let cookie = "FRFTSIHBDTXMKRLZKMNJ";
    let challenge = 3115568843u32;
    let digest = compute_digest(challenge, cookie);

    let expected = [
        0xae, 0x60, 0x9a, 0x74, 0x01, 0x4b, 0x75, 0xac, 0x23, 0x77, 0x0f, 0xa7, 0xa0, 0xc9, 0xc9,
        0x7c,
    ];

    assert_eq!(
        digest, expected,
        "Digest mismatch for challenge={} cookie={}",
        challenge, cookie
    );
}

#[test]
fn test_compute_digest_consistency() {
    let challenge = 123456789u32;
    let cookie = "secret_cookie";

    let digest1 = compute_digest(challenge, cookie);
    let digest2 = compute_digest(challenge, cookie);

    assert_eq!(digest1, digest2);
}

#[test]
fn test_compute_digest_different_cookies() {
    let challenge = 123456789u32;

    let digest1 = compute_digest(challenge, "cookie1");
    let digest2 = compute_digest(challenge, "cookie2");

    assert_ne!(digest1, digest2);
}

#[test]
fn test_compute_digest_different_challenges() {
    let cookie = "secret_cookie";

    let digest1 = compute_digest(123456789, cookie);
    let digest2 = compute_digest(987654321, cookie);

    assert_ne!(digest1, digest2);
}

//
// Challenge Generation
//

#[test]
fn test_generate_challenge_non_zero() {
    let challenge = generate_challenge();
    assert_ne!(challenge, 0);
}

#[test]
fn test_generate_challenge_unique() {
    let challenge1 = generate_challenge();
    thread::sleep(Duration::from_nanos(100));
    let challenge2 = generate_challenge();

    assert_ne!(challenge1, challenge2);
}
