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

use erltf_serde::{to_bytes, to_term};

#[test]
fn test_serialize_large_u64_as_bigint() {
    let val: u64 = u64::MAX;
    let term = to_term(&val).unwrap();

    match term {
        erltf::term::OwnedTerm::BigInt(ref big) => {
            assert!(big.sign.is_positive());
        }
        _ => panic!("Expected BigInt, got {:?}", term),
    }

    let bytes = to_bytes(&val).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn test_serialize_u64_within_i64_range() {
    let val: u64 = i64::MAX as u64;
    let term = to_term(&val).unwrap();

    match term {
        erltf::term::OwnedTerm::Integer(i) => {
            assert_eq!(i, i64::MAX);
        }
        _ => panic!("Expected Integer, got {:?}", term),
    }
}
