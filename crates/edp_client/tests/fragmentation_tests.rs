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

use edp_client::fragmentation::FragmentAssembler;
use std::thread;
use std::time::Duration;

//
// Basic Fragment Assembly
//

#[test]
fn test_single_fragment_message() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 1;
    let fragment_id = 1;
    let payload = vec![1, 2, 3, 4, 5];

    let result = assembler.start_fragment(sequence_id, fragment_id, None, payload.clone());

    assert!(result.is_some());
    let reassembled = result.unwrap();
    assert_eq!(reassembled, payload);
}

#[test]
fn test_two_fragment_message() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 1;
    let payload1 = vec![1, 2, 3];
    let payload2 = vec![4, 5, 6];

    let result1 = assembler.start_fragment(sequence_id, 2, None, payload2.clone());
    assert!(result1.is_none());

    let result2 = assembler.add_fragment(sequence_id, 1, payload1.clone());
    assert!(result2.is_some());

    let reassembled = result2.unwrap();
    let mut expected = Vec::new();
    expected.extend_from_slice(&payload1);
    expected.extend_from_slice(&payload2);
    assert_eq!(reassembled, expected);
}

#[test]
fn test_three_fragment_message() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 42;
    let payload1 = vec![1, 2];
    let payload2 = vec![3, 4];
    let payload3 = vec![5, 6];

    let result1 = assembler.start_fragment(sequence_id, 3, None, payload3.clone());
    assert!(result1.is_none());

    let result2 = assembler.add_fragment(sequence_id, 2, payload2.clone());
    assert!(result2.is_none());

    let result3 = assembler.add_fragment(sequence_id, 1, payload1.clone());
    assert!(result3.is_some());

    let reassembled = result3.unwrap();
    let mut expected = Vec::new();
    expected.extend_from_slice(&payload1);
    expected.extend_from_slice(&payload2);
    expected.extend_from_slice(&payload3);
    assert_eq!(reassembled, expected);
}

//
// Multiple Sequence Handling
//

#[test]
fn test_multiple_sequences() {
    let mut assembler = FragmentAssembler::new();

    let seq1_payload1 = vec![1, 2];
    let seq1_payload2 = vec![3, 4];
    let seq2_payload1 = vec![10, 20];
    let seq2_payload2 = vec![30, 40];

    assembler.start_fragment(1, 2, None, seq1_payload2.clone());
    assembler.start_fragment(2, 2, None, seq2_payload2.clone());

    let result1 = assembler.add_fragment(1, 1, seq1_payload1.clone());
    assert!(result1.is_some());

    let result2 = assembler.add_fragment(2, 1, seq2_payload1.clone());
    assert!(result2.is_some());

    let reassembled1 = result1.unwrap();
    let mut expected1 = Vec::new();
    expected1.extend_from_slice(&seq1_payload1);
    expected1.extend_from_slice(&seq1_payload2);
    assert_eq!(reassembled1, expected1);

    let reassembled2 = result2.unwrap();
    let mut expected2 = Vec::new();
    expected2.extend_from_slice(&seq2_payload1);
    expected2.extend_from_slice(&seq2_payload2);
    assert_eq!(reassembled2, expected2);
}

//
// Atom Cache Support
//

#[test]
fn test_fragment_with_atom_cache() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 1;
    let atom_cache_data = vec![131, 100, 0, 2, 111, 107];
    let payload = vec![1, 2, 3];

    let result = assembler.start_fragment(
        sequence_id,
        1,
        Some(atom_cache_data.clone()),
        payload.clone(),
    );

    assert!(result.is_some());
    let reassembled = result.unwrap();
    let mut expected = Vec::new();
    expected.extend_from_slice(&atom_cache_data);
    expected.extend_from_slice(&payload);
    assert_eq!(reassembled, expected);
}

//
// Assembler Management
//

#[test]
fn test_clear_assembler() {
    let mut assembler = FragmentAssembler::new();

    assembler.start_fragment(1, 2, None, vec![1, 2]);
    assembler.start_fragment(2, 2, None, vec![3, 4]);

    assert_eq!(assembler.pending_count(), 2);

    assembler.clear();

    assert_eq!(assembler.pending_count(), 0);

    let result = assembler.add_fragment(1, 1, vec![5, 6]);
    assert!(result.is_none());
}

//
// Out of Order Fragment Handling
//

#[test]
fn test_out_of_order_fragments() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 1;
    let payload1 = vec![1, 2, 3];
    let payload2 = vec![4, 5, 6];
    let payload3 = vec![7, 8, 9];
    let payload4 = vec![10, 11, 12];

    let result1 = assembler.start_fragment(sequence_id, 4, None, payload4.clone());
    assert!(result1.is_none());

    let result2 = assembler.add_fragment(sequence_id, 1, payload1.clone());
    assert!(result2.is_none());

    let result3 = assembler.add_fragment(sequence_id, 3, payload3.clone());
    assert!(result3.is_none());

    let result4 = assembler.add_fragment(sequence_id, 2, payload2.clone());
    assert!(result4.is_some());

    let reassembled = result4.unwrap();
    let mut expected = Vec::new();
    expected.extend_from_slice(&payload1);
    expected.extend_from_slice(&payload2);
    expected.extend_from_slice(&payload3);
    expected.extend_from_slice(&payload4);
    assert_eq!(reassembled, expected);
}

//
// Fragment Timeout Behavior
//

#[test]
fn test_fragment_timeout() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(100));

    assembler.start_fragment(1, 3, None, vec![1, 2, 3]);
    assembler.start_fragment(2, 2, None, vec![4, 5, 6]);

    assert_eq!(assembler.pending_count(), 2);

    thread::sleep(Duration::from_millis(150));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 2);
    assert_eq!(assembler.pending_count(), 0);
}

#[test]
fn test_fragment_timeout_no_expiry() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_secs(10));

    assembler.start_fragment(1, 3, None, vec![1, 2, 3]);
    assert_eq!(assembler.pending_count(), 1);

    thread::sleep(Duration::from_millis(50));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 0);
    assert_eq!(assembler.pending_count(), 1);
}

#[test]
fn test_fragment_mixed_expiration() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(100));

    assembler.start_fragment(1, 3, None, vec![1]);
    assembler.start_fragment(2, 2, None, vec![2]);

    assert_eq!(assembler.pending_count(), 2);

    thread::sleep(Duration::from_millis(60));

    assembler.start_fragment(3, 2, None, vec![3]);
    assert_eq!(assembler.pending_count(), 3);

    thread::sleep(Duration::from_millis(60));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 2);
    assert_eq!(assembler.pending_count(), 1);

    let result = assembler.add_fragment(3, 1, vec![4]);
    assert!(result.is_some());
    assert_eq!(assembler.pending_count(), 0);
}

//
// Reverse Order Assembly
//

#[test]
fn test_reverse_order_fragments() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 100;
    let payload1 = vec![10, 20];
    let payload2 = vec![30, 40];
    let payload3 = vec![50, 60];

    let result1 = assembler.start_fragment(sequence_id, 3, None, payload3.clone());
    assert!(result1.is_none());

    let result2 = assembler.add_fragment(sequence_id, 2, payload2.clone());
    assert!(result2.is_none());

    let result3 = assembler.add_fragment(sequence_id, 1, payload1.clone());
    assert!(result3.is_some());

    let reassembled = result3.unwrap();
    let mut expected = Vec::new();
    expected.extend_from_slice(&payload1);
    expected.extend_from_slice(&payload2);
    expected.extend_from_slice(&payload3);
    assert_eq!(reassembled, expected);
}

//
// Edge Cases
//

#[test]
fn test_continuation_before_header() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 42;
    let payload1 = vec![1, 2, 3];
    let payload2 = vec![4, 5, 6];
    let payload3 = vec![7, 8, 9];

    let result1 = assembler.add_fragment(sequence_id, 2, payload2.clone());
    assert!(result1.is_none());
    assert_eq!(assembler.pending_count(), 1);

    let result2 = assembler.add_fragment(sequence_id, 1, payload1.clone());
    assert!(result2.is_none());
    assert_eq!(assembler.pending_count(), 1);

    let result3 = assembler.start_fragment(sequence_id, 3, None, payload3.clone());
    assert!(result3.is_some());

    let reassembled = result3.unwrap();
    let mut expected = Vec::new();
    expected.extend_from_slice(&payload1);
    expected.extend_from_slice(&payload2);
    expected.extend_from_slice(&payload3);
    assert_eq!(reassembled, expected);
}

//
// Large Scale Fragment Assembly
//

#[test]
fn test_large_fragment_count() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 1;
    let num_fragments = 100;
    let mut payloads = Vec::new();

    for i in 0..num_fragments {
        payloads.push(vec![i as u8, (i + 1) as u8]);
    }

    let result = assembler.start_fragment(
        sequence_id,
        num_fragments,
        None,
        payloads[(num_fragments - 1) as usize].clone(),
    );
    assert!(result.is_none());

    for i in (1..num_fragments).rev() {
        let result = assembler.add_fragment(sequence_id, i, payloads[(i - 1) as usize].clone());
        if i == 1 {
            assert!(result.is_some());
            let reassembled = result.unwrap();
            let mut expected = Vec::new();
            for payload in &payloads {
                expected.extend_from_slice(payload);
            }
            assert_eq!(reassembled, expected);
        } else {
            assert!(result.is_none());
        }
    }
}

#[test]
fn test_very_large_out_of_order_fragments() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 999;
    let num_fragments = 150;
    let mut payloads = Vec::new();

    for i in 0..num_fragments {
        payloads.push(vec![(i % 256) as u8]);
    }

    assembler.start_fragment(
        sequence_id,
        num_fragments,
        None,
        payloads[(num_fragments - 1) as usize].clone(),
    );

    let mut fragment_ids: Vec<u64> = (1..num_fragments).collect();
    fragment_ids.reverse();

    for (idx, &frag_id) in fragment_ids.iter().enumerate() {
        if idx % 3 == 0 {
            continue;
        }
        assembler.add_fragment(
            sequence_id,
            frag_id,
            payloads[(frag_id - 1) as usize].clone(),
        );
    }

    for (idx, &frag_id) in fragment_ids.iter().enumerate() {
        if idx % 3 != 0 {
            continue;
        }
        let result = assembler.add_fragment(
            sequence_id,
            frag_id,
            payloads[(frag_id - 1) as usize].clone(),
        );

        if frag_id == 1 {
            assert!(result.is_some());
            let reassembled = result.unwrap();
            let mut expected = Vec::new();
            for payload in &payloads {
                expected.extend_from_slice(payload);
            }
            assert_eq!(reassembled, expected);
        }
    }
}

#[test]
fn test_fragments_expire_after_timeout() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(10));

    assembler.start_fragment(100, 3, None, vec![1, 2, 3]);
    assembler.add_fragment(100, 2, vec![4, 5, 6]);

    assert_eq!(assembler.pending_count(), 1);

    thread::sleep(Duration::from_millis(20));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 1, "Should have expired 1 sequence");
    assert_eq!(assembler.pending_count(), 0);
}

#[test]
fn test_incomplete_fragments_can_timeout() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(10));

    assembler.start_fragment(200, 5, None, vec![1]);
    assembler.add_fragment(200, 4, vec![2]);

    assert_eq!(assembler.pending_count(), 1);

    thread::sleep(Duration::from_millis(20));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 1);
}

#[test]
fn test_multiple_sequences_timeout_independently() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(50));

    assembler.start_fragment(1, 2, None, vec![1]);
    thread::sleep(Duration::from_millis(20));
    assembler.start_fragment(2, 2, None, vec![2]);

    assert_eq!(assembler.pending_count(), 2);

    thread::sleep(Duration::from_millis(35));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 1, "Only first sequence should have expired");
    assert_eq!(assembler.pending_count(), 1);
}

#[test]
fn test_complete_fragments_before_timeout() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(100));

    assembler.start_fragment(300, 2, None, vec![1, 2, 3]);
    let result = assembler.add_fragment(300, 1, vec![4, 5, 6]);

    assert!(result.is_some(), "Should complete before timeout");
    assert_eq!(assembler.pending_count(), 0);
}

#[test]
fn test_cleanup_does_not_affect_recent_fragments() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(100));

    assembler.start_fragment(400, 3, None, vec![1]);

    thread::sleep(Duration::from_millis(10));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 0, "Recent fragments should not expire");
    assert_eq!(assembler.pending_count(), 1);
}

#[test]
fn test_adding_fragment_updates_timestamp() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(50));

    assembler.start_fragment(500, 3, None, vec![1]);
    thread::sleep(Duration::from_millis(30));

    assembler.add_fragment(500, 2, vec![2]);
    thread::sleep(Duration::from_millis(30));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 0, "Timestamp should be updated by add_fragment");
    assert_eq!(assembler.pending_count(), 1);
}

//
// Duplicate Fragment Handling
//

#[test]
fn test_duplicate_fragment_ignored() {
    let mut assembler = FragmentAssembler::new();

    let sequence_id = 1;
    let payload1 = vec![1, 2, 3];
    let payload2 = vec![4, 5, 6];
    let duplicate_payload = vec![99, 99, 99];

    let result = assembler.start_fragment(sequence_id, 2, None, payload2.clone());
    assert!(result.is_none());

    let result = assembler.add_fragment(sequence_id, 1, payload1.clone());
    assert!(result.is_some(), "Should complete with first fragment");

    let reassembled = result.unwrap();
    let mut expected = Vec::new();
    expected.extend_from_slice(&payload1);
    expected.extend_from_slice(&payload2);
    assert_eq!(reassembled, expected);

    let sequence_id = 2;
    assembler.start_fragment(sequence_id, 3, None, vec![1]);
    assembler.add_fragment(sequence_id, 2, vec![2]);
    assembler.add_fragment(sequence_id, 2, duplicate_payload);

    let result = assembler.add_fragment(sequence_id, 1, vec![0]);
    assert!(result.is_some(), "Should ignore duplicate fragment 2");
    let reassembled = result.unwrap();
    assert_eq!(
        reassembled,
        vec![0, 2, 1],
        "Should use original fragment, not duplicate"
    );
}

#[test]
fn test_fragment_id_zero_ignored() {
    let mut assembler = FragmentAssembler::new();

    let result = assembler.start_fragment(1, 0, None, vec![1, 2, 3]);
    assert!(
        result.is_none(),
        "Fragment ID 0 in header should be ignored"
    );

    assembler.start_fragment(2, 2, None, vec![4, 5, 6]);
    let result = assembler.add_fragment(2, 0, vec![99]);
    assert!(result.is_none(), "Fragment ID 0 should be ignored");
}

#[test]
fn test_very_large_fragment_count() {
    let mut assembler = FragmentAssembler::new();

    let large_count = u64::MAX;
    let result = assembler.start_fragment(1, large_count, None, vec![1]);
    assert!(result.is_none(), "Should reject invalid fragment counts");
    assert_eq!(
        assembler.pending_count(),
        0,
        "Should not buffer invalid counts"
    );
}

#[test]
fn test_fragment_count_at_vec_limit() {
    let mut assembler = FragmentAssembler::new();

    let limit = 100_000u64;
    let result = assembler.start_fragment(1, limit, None, vec![255]);
    assert!(result.is_none(), "Should not complete immediately");
    assert_eq!(assembler.pending_count(), 1);

    for i in (1..limit).rev() {
        let result = assembler.add_fragment(1, i, vec![i as u8]);
        if i == 1 {
            assert!(
                result.is_some(),
                "Should complete when all fragments received"
            );
        } else {
            assert!(result.is_none());
        }
    }
}

#[test]
fn test_fragment_count_exceeds_vec_limit() {
    let mut assembler = FragmentAssembler::new();

    let count_over_limit = 100_001u64;
    let result = assembler.start_fragment(1, count_over_limit, None, vec![1]);
    assert!(result.is_none());
    assert_eq!(
        assembler.pending_count(),
        1,
        "Should use HashMap for large counts"
    );
}

#[test]
fn test_fragment_id_beyond_total_fragments() {
    let mut assembler = FragmentAssembler::new();

    assembler.start_fragment(1, 5, None, vec![5]);

    assembler.add_fragment(1, 10, vec![99]);

    assembler.add_fragment(1, 4, vec![4]);
    assembler.add_fragment(1, 3, vec![3]);
    assembler.add_fragment(1, 2, vec![2]);

    let result = assembler.add_fragment(1, 1, vec![1]);
    assert!(
        result.is_some(),
        "Should complete despite out-of-range fragment"
    );
}

#[test]
fn test_storage_transition_vec_to_hashmap() {
    let mut assembler = FragmentAssembler::new();

    assembler.add_fragment(1, 50, vec![50]);
    assembler.add_fragment(1, 100, vec![100]);

    let large_count = 100_001u64;
    let result = assembler.start_fragment(1, large_count, None, vec![255]);
    assert!(result.is_none());

    assert_eq!(assembler.pending_count(), 1);
}

#[test]
fn test_cleanup_after_timeout_frees_memory() {
    let mut assembler = FragmentAssembler::with_timeout(Duration::from_millis(10));

    for i in 0..10 {
        assembler.start_fragment(i, 100, None, vec![1, 2, 3]);
    }

    assert_eq!(assembler.pending_count(), 10);

    thread::sleep(Duration::from_millis(20));

    let expired = assembler.cleanup_expired();
    assert_eq!(expired, 10, "All sequences should have expired");
    assert_eq!(assembler.pending_count(), 0, "Memory should be freed");
}
