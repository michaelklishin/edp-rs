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

use edp_client::PidAllocator;
use erltf::types::Atom;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::Ordering;

const MAX_PROCESSES_PER_NODE: u32 = 1_048_576;

//
// Sequential Allocation Tests
//

#[test]
fn test_allocate_sequential() {
    let allocator = PidAllocator::new(Atom::new("test@localhost"), 1);

    let pid1 = allocator.allocate().unwrap();
    assert_eq!(pid1.id, 1);
    assert_eq!(pid1.serial, 0);
    assert_eq!(pid1.creation, 1);
    assert_eq!(pid1.node.as_str(), "test@localhost");

    let pid2 = allocator.allocate().unwrap();
    assert_eq!(pid2.id, 2);
    assert_eq!(pid2.serial, 0);
    assert_eq!(pid2.creation, 1);
}

#[test]
fn test_allocate_wraparound() {
    let allocator = PidAllocator::new(Atom::new("test@localhost"), 1);

    allocator
        .next_id_test_only()
        .store(MAX_PROCESSES_PER_NODE - 1, Ordering::Relaxed);

    let pid1 = allocator.allocate().unwrap();
    assert_eq!(pid1.id, MAX_PROCESSES_PER_NODE - 1);
    assert_eq!(pid1.serial, 0);

    let pid2 = allocator.allocate().unwrap();
    assert_eq!(pid2.id, MAX_PROCESSES_PER_NODE);

    let pid3 = allocator.allocate().unwrap();
    assert_eq!(pid3.id, 1);
    assert_eq!(pid3.serial, 1);
}

//
// Concurrent Allocation Tests
//

#[tokio::test]
async fn test_concurrent_pid_allocation() {
    let allocator = Arc::new(PidAllocator::new(Atom::new("test@localhost"), 1));
    let mut handles = vec![];

    for _ in 0..10 {
        let allocator = allocator.clone();
        let handle = tokio::spawn(async move {
            let mut pids = Vec::new();
            for _ in 0..100 {
                pids.push(allocator.allocate().unwrap());
            }
            pids
        });
        handles.push(handle);
    }

    let mut all_pids = HashSet::new();
    for handle in handles {
        let pids = handle.await.unwrap();
        for pid in pids {
            let key = (pid.id, pid.serial);
            assert!(
                !all_pids.contains(&key),
                "Duplicate PID allocated: {:?}",
                pid
            );
            all_pids.insert(key);
        }
    }

    assert_eq!(all_pids.len(), 1000);
}

#[test]
fn test_wraparound_increments_serial() {
    let allocator = PidAllocator::new(Atom::new("test@localhost"), 1);

    allocator
        .next_id_test_only()
        .store(1_048_575, Ordering::SeqCst);

    let pid1 = allocator.allocate().unwrap();
    assert_eq!(pid1.id, 1_048_575);
    assert_eq!(pid1.serial, 0);

    let pid2 = allocator.allocate().unwrap();
    assert_eq!(pid2.id, 1_048_576);
    assert_eq!(pid2.serial, 1);

    let pid3 = allocator.allocate().unwrap();
    assert_eq!(pid3.id, 1);
    assert_eq!(pid3.serial, 1);
}

#[test]
fn test_pid_wraps_at_max_processes() {
    let allocator = PidAllocator::new(Atom::new("node@host"), 1);

    let max_id = 1_048_576u32;
    allocator
        .next_id_test_only()
        .store(max_id - 1, Ordering::Relaxed);

    let pid1 = allocator.allocate().unwrap();
    assert_eq!(pid1.id, max_id - 1);
    assert_eq!(pid1.serial, 0);

    let pid2 = allocator.allocate().unwrap();
    assert_eq!(pid2.id, max_id, "Allocates max_id with incremented serial");
    assert_eq!(
        pid2.serial, 1,
        "Serial increments when id >= MAX_PROCESSES_PER_NODE"
    );

    let pid3 = allocator.allocate().unwrap();
    assert_eq!(pid3.id, 1, "PID wraps to 1 after max_id");
    assert_eq!(pid3.serial, 1, "Serial remains at 1 after wrap");
}

#[test]
fn test_pid_does_not_allocate_zero() {
    let allocator = PidAllocator::new(Atom::new("node@host"), 1);

    for _ in 0..10 {
        let pid = allocator.allocate().unwrap();
        assert_ne!(pid.id, 0, "PID ID should never be 0");
    }
}

#[test]
fn test_pid_serial_increments_on_each_wrap() {
    let allocator = PidAllocator::new(Atom::new("node@host"), 1);

    let max_id = 1_048_576u32;
    allocator
        .next_id_test_only()
        .store(max_id, Ordering::Relaxed);

    let pid1 = allocator.allocate().unwrap();
    assert_eq!(pid1.id, max_id, "First wrap allocates max_id");
    assert_eq!(pid1.serial, 1, "Serial increments on first wrap");

    allocator
        .next_id_test_only()
        .store(max_id, Ordering::Relaxed);

    let pid2 = allocator.allocate().unwrap();
    assert_eq!(pid2.id, max_id, "Second wrap allocates max_id again");
    assert_eq!(pid2.serial, 2, "Serial increments on second wrap");
}

#[test]
fn test_pid_creation_value_persists() {
    let allocator = PidAllocator::new(Atom::new("node@host"), 42);

    for _ in 0..5 {
        let pid = allocator.allocate().unwrap();
        assert_eq!(pid.creation, 42);
    }
}

#[test]
fn test_pid_set_creation_updates_value() {
    let allocator = PidAllocator::new(Atom::new("node@host"), 1);

    let pid1 = allocator.allocate().unwrap();
    assert_eq!(pid1.creation, 1);

    allocator.set_creation(5);

    let pid2 = allocator.allocate().unwrap();
    assert_eq!(pid2.creation, 5);
}

#[test]
fn test_pid_serial_overflow_wraps_correctly() {
    let allocator = PidAllocator::new(Atom::new("node@host"), 1);

    for i in 0..5 {
        allocator
            .next_id_test_only()
            .store(MAX_PROCESSES_PER_NODE, Ordering::Relaxed);

        let serial_value = u32::MAX as u64 - 2 + i;
        allocator
            .next_serial_test_only()
            .store(serial_value, Ordering::Relaxed);

        let pid = allocator.allocate().unwrap();

        let expected_serial = ((serial_value + 1) % (u32::MAX as u64 + 1)) as u32;
        assert_eq!(
            pid.serial, expected_serial,
            "Serial should wrap at u32::MAX boundary (iteration {})",
            i
        );
    }
}
