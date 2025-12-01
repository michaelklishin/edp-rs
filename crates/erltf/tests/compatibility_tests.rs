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

use erltf::types::{Atom, ExternalPid, ExternalPort, ExternalReference};
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

// ============================================================================
// LOCAL_EXT Roundtrip Tests
// ============================================================================

/// Helper to construct LOCAL_EXT bytes wrapping a NEW_PID_EXT (tag 88)
fn make_local_ext_pid_bytes() -> Vec<u8> {
    // LOCAL_EXT format: version(131) + LOCAL_EXT(121) + hash(8 bytes) + nested term
    // Nested term: NEW_PID_EXT(88) + atom + id(4) + serial(4) + creation(4)
    let mut bytes = vec![
        131, // version tag
        121, // LOCAL_EXT tag
        // 8 bytes hash (arbitrary values for testing)
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE,
        // NEW_PID_EXT
        88,  // NEW_PID_EXT tag
        119, // SMALL_ATOM_UTF8_EXT tag
        14,  // atom length ("test@localhost" = 14 chars)
    ];
    bytes.extend_from_slice(b"test@localhost"); // node name
    bytes.extend_from_slice(&[0, 0, 0, 42]); // id = 42
    bytes.extend_from_slice(&[0, 0, 0, 7]); // serial = 7
    bytes.extend_from_slice(&[0, 0, 0, 1]); // creation = 1
    bytes
}

/// Helper to construct LOCAL_EXT bytes wrapping a V4_PORT_EXT (tag 120)
fn make_local_ext_port_bytes() -> Vec<u8> {
    let mut bytes = vec![
        131, // version tag
        121, // LOCAL_EXT tag
        // 8 bytes hash
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
        // V4_PORT_EXT
        120, // V4_PORT_EXT tag
        119, // SMALL_ATOM_UTF8_EXT tag
        14,  // atom length ("test@localhost" = 14 chars)
    ];
    bytes.extend_from_slice(b"test@localhost"); // node name
    bytes.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 99]); // id = 99 (8 bytes)
    bytes.extend_from_slice(&[0, 0, 0, 2]); // creation = 2
    bytes
}

/// Helper to construct LOCAL_EXT bytes wrapping a NEWER_REFERENCE_EXT (tag 90)
fn make_local_ext_reference_bytes() -> Vec<u8> {
    let mut bytes = vec![
        131, // version tag
        121, // LOCAL_EXT tag
        // 8 bytes hash
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11,
        // NEWER_REFERENCE_EXT
        90, // NEWER_REFERENCE_EXT tag
        0, 3,   // len = 3 (number of id words)
        119, // SMALL_ATOM_UTF8_EXT tag
        14,  // atom length ("test@localhost" = 14 chars)
    ];
    bytes.extend_from_slice(b"test@localhost"); // node name
    bytes.extend_from_slice(&[0, 0, 0, 3]); // creation = 3
    bytes.extend_from_slice(&[0, 0, 0, 1]); // id[0] = 1
    bytes.extend_from_slice(&[0, 0, 0, 2]); // id[1] = 2
    bytes.extend_from_slice(&[0, 0, 0, 3]); // id[2] = 3
    bytes
}

#[test]
fn test_local_ext_pid_decodes_with_is_local_ext_true() {
    let bytes = make_local_ext_pid_bytes();
    let term = decode(&bytes).expect("Failed to decode LOCAL_EXT PID");

    if let OwnedTerm::Pid(pid) = term {
        assert!(
            pid.is_local_ext(),
            "PID decoded from LOCAL_EXT should have is_local_ext() == true"
        );
        assert_eq!(pid.node, Atom::new("test@localhost"));
        assert_eq!(pid.id, 42);
        assert_eq!(pid.serial, 7);
        assert_eq!(pid.creation, 1);
    } else {
        panic!("Expected Pid, got {:?}", term);
    }
}

#[test]
fn test_local_ext_port_decodes_with_is_local_ext_true() {
    let bytes = make_local_ext_port_bytes();
    let term = decode(&bytes).expect("Failed to decode LOCAL_EXT Port");

    if let OwnedTerm::Port(port) = term {
        assert!(
            port.is_local_ext(),
            "Port decoded from LOCAL_EXT should have is_local_ext() == true"
        );
        assert_eq!(port.node, Atom::new("test@localhost"));
        assert_eq!(port.id, 99);
        assert_eq!(port.creation, 2);
    } else {
        panic!("Expected Port, got {:?}", term);
    }
}

#[test]
fn test_local_ext_reference_decodes_with_is_local_ext_true() {
    let bytes = make_local_ext_reference_bytes();
    let term = decode(&bytes).expect("Failed to decode LOCAL_EXT Reference");

    if let OwnedTerm::Reference(reference) = term {
        assert!(
            reference.is_local_ext(),
            "Reference decoded from LOCAL_EXT should have is_local_ext() == true"
        );
        assert_eq!(reference.node, Atom::new("test@localhost"));
        assert_eq!(reference.creation, 3);
        assert_eq!(reference.ids, vec![1, 2, 3]);
    } else {
        panic!("Expected Reference, got {:?}", term);
    }
}

#[test]
fn test_local_ext_pid_roundtrip_produces_identical_bytes() {
    let original_bytes = make_local_ext_pid_bytes();
    let term = decode(&original_bytes).expect("Failed to decode LOCAL_EXT PID");

    let reencoded = encode(&term).expect("Failed to re-encode LOCAL_EXT PID");

    assert_eq!(
        original_bytes, reencoded,
        "Re-encoded LOCAL_EXT PID should be identical to original bytes"
    );
}

#[test]
fn test_local_ext_port_roundtrip_produces_identical_bytes() {
    let original_bytes = make_local_ext_port_bytes();
    let term = decode(&original_bytes).expect("Failed to decode LOCAL_EXT Port");

    let reencoded = encode(&term).expect("Failed to re-encode LOCAL_EXT Port");

    assert_eq!(
        original_bytes, reencoded,
        "Re-encoded LOCAL_EXT Port should be identical to original bytes"
    );
}

#[test]
fn test_local_ext_reference_roundtrip_produces_identical_bytes() {
    let original_bytes = make_local_ext_reference_bytes();
    let term = decode(&original_bytes).expect("Failed to decode LOCAL_EXT Reference");

    let reencoded = encode(&term).expect("Failed to re-encode LOCAL_EXT Reference");

    assert_eq!(
        original_bytes, reencoded,
        "Re-encoded LOCAL_EXT Reference should be identical to original bytes"
    );
}

#[test]
fn test_local_ext_pid_equality_ignores_local_ext_bytes() {
    // A PID with LOCAL_EXT bytes
    let local_ext_bytes = make_local_ext_pid_bytes();
    let local_ext_term = decode(&local_ext_bytes).expect("Failed to decode LOCAL_EXT PID");

    // A PID without LOCAL_EXT bytes (same logical PID)
    let regular_pid = ExternalPid::new(Atom::new("test@localhost"), 42, 7, 1);

    if let OwnedTerm::Pid(local_ext_pid) = local_ext_term {
        assert!(local_ext_pid.is_local_ext());
        assert!(!regular_pid.is_local_ext());

        // They should still be equal
        assert_eq!(
            local_ext_pid, regular_pid,
            "PIDs should be equal regardless of local_ext_bytes"
        );
    } else {
        panic!("Expected Pid");
    }
}

#[test]
fn test_local_ext_port_equality_ignores_local_ext_bytes() {
    let local_ext_bytes = make_local_ext_port_bytes();
    let local_ext_term = decode(&local_ext_bytes).expect("Failed to decode LOCAL_EXT Port");

    let regular_port = ExternalPort::new(Atom::new("test@localhost"), 99, 2);

    if let OwnedTerm::Port(local_ext_port) = local_ext_term {
        assert!(local_ext_port.is_local_ext());
        assert!(!regular_port.is_local_ext());

        assert_eq!(
            local_ext_port, regular_port,
            "Ports should be equal regardless of local_ext_bytes"
        );
    } else {
        panic!("Expected Port");
    }
}

#[test]
fn test_local_ext_reference_equality_ignores_local_ext_bytes() {
    let local_ext_bytes = make_local_ext_reference_bytes();
    let local_ext_term = decode(&local_ext_bytes).expect("Failed to decode LOCAL_EXT Reference");

    let regular_reference = ExternalReference::new(Atom::new("test@localhost"), 3, vec![1, 2, 3]);

    if let OwnedTerm::Reference(local_ext_ref) = local_ext_term {
        assert!(local_ext_ref.is_local_ext());
        assert!(!regular_reference.is_local_ext());

        assert_eq!(
            local_ext_ref, regular_reference,
            "References should be equal regardless of local_ext_bytes"
        );
    } else {
        panic!("Expected Reference");
    }
}

#[test]
fn test_new_pid_ext_round_trip() {
    // A PID constructed without decoding should have no raw bytes
    let regular_pid = ExternalPid::new(Atom::new("node@host"), 1, 0, 1);
    assert!(
        !regular_pid.is_local_ext(),
        "Regularly constructed PID should have is_local_ext() == false"
    );

    // Encode and decode a regular PID - NEW_PID_EXT doesn't preserve raw bytes
    // because it can be exactly reconstructed from parsed fields
    let encoded = encode(&OwnedTerm::Pid(regular_pid.clone())).unwrap();
    let decoded = decode(&encoded).unwrap();

    if let OwnedTerm::Pid(decoded_pid) = decoded {
        // NEW_PID_EXT does not preserve raw bytes (unlike LOCAL_EXT)
        // because it can be exactly reconstructed from (node, id, serial, creation)
        assert!(
            !decoded_pid.is_local_ext(),
            "PID decoded from NEW_PID_EXT should not be local_ext"
        );
        // Ensure PIDs are equal
        assert_eq!(
            regular_pid, decoded_pid,
            "PIDs should be equal after round-trip"
        );
        // Verify round-trip produces identical bytes
        let re_encoded = encode(&OwnedTerm::Pid(decoded_pid)).unwrap();
        assert_eq!(
            encoded, re_encoded,
            "Round-trip should produce identical bytes"
        );
    } else {
        panic!("Expected Pid");
    }
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
