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

use edp_client::connection::Connection;
use erltf::decoder::AtomCache;

#[test]
fn test_decode_fragment_with_invalid_version_tag() {
    let mut cache = AtomCache::new();
    let corrupted_data = vec![255, 68, 1, 2, 3];

    let result = Connection::decode_complete_fragment(&corrupted_data, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_with_invalid_dist_header() {
    let mut cache = AtomCache::new();
    let corrupted_data = vec![131, 255, 1, 2, 3];

    let result = Connection::decode_complete_fragment(&corrupted_data, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_with_truncated_data() {
    let mut cache = AtomCache::new();
    let truncated_data = vec![131];

    let result = Connection::decode_complete_fragment(&truncated_data, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_with_empty_data() {
    let mut cache = AtomCache::new();
    let empty_data = vec![];

    let result = Connection::decode_complete_fragment(&empty_data, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_with_malformed_control_message() {
    let mut cache = AtomCache::new();
    let malformed_data = vec![131, 108, 0, 0, 0, 0, 106];

    let result = Connection::decode_complete_fragment(&malformed_data, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_with_invalid_atom_cache_ref() {
    let mut cache = AtomCache::new();
    let data_with_bad_cache_ref = vec![
        131, 68, 1, 82, 255, 108, 0, 0, 0, 1, 100, 0, 4, 115, 101, 110, 100, 106,
    ];

    let result = Connection::decode_complete_fragment(&data_with_bad_cache_ref, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_missing_message_payload() {
    let mut cache = AtomCache::new();
    let valid_control_only = vec![131, 100, 0, 4, 112, 105, 110, 103];

    let result = Connection::decode_complete_fragment(&valid_control_only, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_with_oversized_list() {
    let mut cache = AtomCache::new();
    let mut oversized_list = vec![131, 108];
    oversized_list.extend_from_slice(&u32::MAX.to_be_bytes());
    oversized_list.extend_from_slice(&[100, 0, 1, 97, 106]);

    let result = Connection::decode_complete_fragment(&oversized_list, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_with_corrupted_atom_length() {
    let mut cache = AtomCache::new();
    let corrupted_atom = vec![131, 100, 255, 255, 97, 98, 99];

    let result = Connection::decode_complete_fragment(&corrupted_atom, &mut cache);
    assert!(result.is_err());
}

#[test]
fn test_decode_fragment_with_nested_corruption() {
    let mut cache = AtomCache::new();
    let nested_corrupt = vec![
        131, 104, 2, 100, 0, 2, 111, 107, 108, 255, 255, 255, 255, 106,
    ];

    let result = Connection::decode_complete_fragment(&nested_corrupt, &mut cache);
    assert!(result.is_err());
}
