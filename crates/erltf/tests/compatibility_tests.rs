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

use erltf::types::{Atom, ExternalPid};
use erltf::{
    AtomCache, OwnedTerm, decode, decode_with_atom_cache, encode, encode_with_dist_header_multi,
};
use std::io::Write;

// ============================================================================
// Backward Compatibility Tests
// ============================================================================

#[test]
fn test_decode_old_reference_ext() {
    let bytes = vec![
        131, 101, 100, 0, 13, 110, 111, 110, 111, 100, 101, 64, 110, 111, 104, 111, 115, 116, 0, 0,
        0, 1, 0,
    ];
    let term = decode(&bytes).expect("Failed to decode REFERENCE_EXT");
    if let OwnedTerm::Reference(r) = term {
        assert_eq!(r.node, "nonode@nohost");
        assert_eq!(r.ids.len(), 1);
        assert_eq!(r.ids[0], 1);
        assert_eq!(r.creation, 0);
    } else {
        panic!("Expected Reference, got {:?}", term);
    }
}

#[test]
fn test_decode_old_port_ext() {
    let bytes = vec![
        131, 102, 100, 0, 13, 110, 111, 110, 111, 100, 101, 64, 110, 111, 104, 111, 115, 116, 0, 0,
        0, 42, 0,
    ];
    let term = decode(&bytes).expect("Failed to decode PORT_EXT");
    if let OwnedTerm::Port(p) = term {
        assert_eq!(p.node, "nonode@nohost");
        assert_eq!(p.id, 42);
        assert_eq!(p.creation, 0);
    } else {
        panic!("Expected Port, got {:?}", term);
    }
}

#[test]
fn test_decode_old_pid_ext() {
    let bytes = vec![
        131, 103, 100, 0, 13, 110, 111, 110, 111, 100, 101, 64, 110, 111, 104, 111, 115, 116, 0, 0,
        0, 100, 0, 0, 0, 200, 0,
    ];
    let term = decode(&bytes).expect("Failed to decode PID_EXT");
    if let OwnedTerm::Pid(pid) = term {
        assert_eq!(pid.node, "nonode@nohost");
        assert_eq!(pid.id, 100);
        assert_eq!(pid.serial, 200);
        assert_eq!(pid.creation, 0);
    } else {
        panic!("Expected Pid, got {:?}", term);
    }
}

#[test]
fn test_decode_new_reference_ext() {
    let bytes = vec![
        131, 114, 0, 3, 100, 0, 13, 110, 111, 110, 111, 100, 101, 64, 110, 111, 104, 111, 115, 116,
        0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3,
    ];
    let term = decode(&bytes).expect("Failed to decode NEW_REFERENCE_EXT");
    if let OwnedTerm::Reference(r) = term {
        assert_eq!(r.node, "nonode@nohost");
        assert_eq!(r.ids.len(), 3);
        assert_eq!(r.ids, vec![1, 2, 3]);
        assert_eq!(r.creation, 0);
    } else {
        panic!("Expected Reference, got {:?}", term);
    }
}

#[test]
fn test_decode_local_ext_wraps_atom() {
    let bytes = vec![
        131, 121, 0, 0, 0, 0, 0, 0, 0, 1, 100, 0, 4, 116, 101, 115, 116,
    ];
    let term = decode(&bytes).expect("Failed to decode LOCAL_EXT");
    assert_eq!(term, OwnedTerm::atom("test"));
}

#[test]
fn test_decode_local_ext_wraps_tuple() {
    let bytes = vec![
        131, 121, 0, 0, 0, 0, 0, 0, 0, 1, 104, 2, 100, 0, 2, 111, 107, 97, 42,
    ];
    let term = decode(&bytes).expect("Failed to decode LOCAL_EXT with tuple");
    assert_eq!(
        term,
        OwnedTerm::tuple(vec![OwnedTerm::atom("ok"), OwnedTerm::Integer(42)])
    );
}

#[test]
fn test_old_formats_roundtrip_as_modern() {
    let old_pid_bytes = vec![
        131, 103, 100, 0, 13, 110, 111, 110, 111, 100, 101, 64, 110, 111, 104, 111, 115, 116, 0, 0,
        0, 100, 0, 0, 0, 200, 0,
    ];
    let pid = decode(&old_pid_bytes).expect("Failed to decode old PID");
    let reencoded = encode(&pid).expect("Failed to re-encode PID");

    let redecoded = decode(&reencoded).expect("Failed to decode re-encoded PID");
    assert_eq!(pid, redecoded);
}

// ============================================================================
// Compression Tests
// ============================================================================

#[test]
fn test_decompress_atom() {
    let uncompressed_atom = OwnedTerm::atom("test");
    let encoded = encode(&uncompressed_atom).expect("Failed to encode atom");

    let mut compressed_bytes = Vec::new();
    compressed_bytes.push(131);
    compressed_bytes.push(80);

    let uncompressed_size = (encoded.len() - 1) as u32;
    compressed_bytes.extend_from_slice(&uncompressed_size.to_be_bytes());

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&encoded[1..]).unwrap();
    let compressed_data = encoder.finish().unwrap();
    compressed_bytes.extend_from_slice(&compressed_data);

    let decoded = decode(&compressed_bytes).expect("Failed to decode compressed atom");
    assert_eq!(decoded, uncompressed_atom);
}

#[test]
fn test_decompress_list() {
    let uncompressed_list = OwnedTerm::list(vec![
        OwnedTerm::Integer(1),
        OwnedTerm::Integer(2),
        OwnedTerm::Integer(3),
    ]);
    let encoded = encode(&uncompressed_list).expect("Failed to encode list");

    let mut compressed_bytes = Vec::new();
    compressed_bytes.push(131);
    compressed_bytes.push(80);

    let uncompressed_size = (encoded.len() - 1) as u32;
    compressed_bytes.extend_from_slice(&uncompressed_size.to_be_bytes());

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&encoded[1..]).unwrap();
    let compressed_data = encoder.finish().unwrap();
    compressed_bytes.extend_from_slice(&compressed_data);

    let decoded = decode(&compressed_bytes).expect("Failed to decode compressed list");
    assert_eq!(decoded, uncompressed_list);
}

#[test]
fn test_decompress_large_binary() {
    let large_data = vec![42u8; 10000];
    let binary_term = OwnedTerm::Binary(large_data.clone());
    let encoded = encode(&binary_term).expect("Failed to encode binary");

    let mut compressed_bytes = Vec::new();
    compressed_bytes.push(131);
    compressed_bytes.push(80);

    let uncompressed_size = (encoded.len() - 1) as u32;
    compressed_bytes.extend_from_slice(&uncompressed_size.to_be_bytes());

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&encoded[1..]).unwrap();
    let compressed_data = encoder.finish().unwrap();
    compressed_bytes.extend_from_slice(&compressed_data);

    let decoded = decode(&compressed_bytes).expect("Failed to decode compressed binary");
    assert_eq!(decoded, binary_term);
}

#[test]
fn test_decompress_nested_structure() {
    let nested = OwnedTerm::tuple(vec![
        OwnedTerm::atom("ok"),
        OwnedTerm::list(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::tuple(vec![
                OwnedTerm::atom("nested"),
                OwnedTerm::Binary(vec![1, 2, 3, 4, 5]),
            ]),
        ]),
    ]);
    let encoded = encode(&nested).expect("Failed to encode nested structure");

    let mut compressed_bytes = Vec::new();
    compressed_bytes.push(131);
    compressed_bytes.push(80);

    let uncompressed_size = (encoded.len() - 1) as u32;
    compressed_bytes.extend_from_slice(&uncompressed_size.to_be_bytes());

    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&encoded[1..]).unwrap();
    let compressed_data = encoder.finish().unwrap();
    compressed_bytes.extend_from_slice(&compressed_data);

    let decoded = decode(&compressed_bytes).expect("Failed to decode compressed nested structure");
    assert_eq!(decoded, nested);
}

// ============================================================================
// Distribution Header Tests
// ============================================================================

#[test]
fn test_dist_header_roundtrip() {
    let control = OwnedTerm::Tuple(vec![
        OwnedTerm::Integer(6), // RegSend
        OwnedTerm::Pid(ExternalPid::new(
            Atom::new("rust_client@sunnyside"),
            1,
            0,
            1762047320,
        )),
        OwnedTerm::Atom(Atom::new("")),
        OwnedTerm::Atom(Atom::new("rex")),
    ]);

    let payload = OwnedTerm::Tuple(vec![
        OwnedTerm::Pid(ExternalPid::new(
            Atom::new("rust_client@sunnyside"),
            1,
            0,
            1762047320,
        )),
        OwnedTerm::Tuple(vec![
            OwnedTerm::Atom(Atom::new("call")),
            OwnedTerm::Atom(Atom::new("rabbit")),
            OwnedTerm::Atom(Atom::new("status")),
            OwnedTerm::List(vec![]),
            OwnedTerm::Atom(Atom::new("user")),
        ]),
    ]);

    let encoded = encode_with_dist_header_multi(&[&control, &payload]).unwrap();

    println!("Encoded length: {}", encoded.len());
    println!(
        "First 100 bytes: {:02x?}",
        &encoded[..encoded.len().min(100)]
    );

    let mut cache = AtomCache::new();
    let (decoded_control, decoded_payload_opt) =
        decode_with_atom_cache(&encoded, &mut cache).unwrap();

    assert_eq!(decoded_control, control, "Control message should match");
    assert!(decoded_payload_opt.is_some(), "Payload should be present");

    let decoded_payload = decoded_payload_opt.unwrap();
    if let OwnedTerm::Tuple(decoded_elements) = &decoded_payload {
        assert_eq!(decoded_elements.len(), 2);
        if let OwnedTerm::Tuple(inner) = &decoded_elements[1] {
            assert_eq!(inner.len(), 5);
            assert!(matches!(&inner[3], OwnedTerm::Nil));
        } else {
            panic!("Expected inner tuple");
        }
    } else {
        panic!("Expected outer tuple, got {:?}", decoded_payload);
    }
}
