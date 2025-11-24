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

use erltf::decoder::AtomCache;
use erltf::types::Atom;

#[test]
fn test_atom_cache_stores_and_retrieves() {
    let mut cache = AtomCache::new();

    cache.insert(0, Atom::new("test"));
    let result = cache.get(0);

    assert!(result.is_some());
    assert_eq!(result.unwrap().as_str(), "test");
}

#[test]
fn test_atom_cache_max_capacity() {
    let mut cache = AtomCache::new();

    for i in 0..256 {
        cache.insert(i as u8, Atom::new(&format!("atom_{}", i)));
    }

    assert_eq!(cache.len(), 256, "Cache holds all 256 possible indices");

    cache.insert(0, Atom::new("replaced"));
    assert_eq!(cache.len(), 256, "Replacing entry doesn't change size");
    assert_eq!(cache.get(0).unwrap().as_str(), "replaced");
}

#[test]
fn test_atom_cache_protocol_limit() {
    let mut cache = AtomCache::new();

    for i in 0..1000 {
        let name = format!("atom_{}", i);
        cache.insert((i % 256) as u8, Atom::new(&name));
    }

    assert_eq!(cache.len(), 256, "Cache limited to 256 entries by u8 index");
}

#[test]
fn test_atom_cache_overwrites_same_index() {
    let mut cache = AtomCache::new();

    cache.insert(5, Atom::new("first"));
    cache.insert(5, Atom::new("second"));

    let result = cache.get(5);
    assert_eq!(result.unwrap().as_str(), "second");
}

#[test]
fn test_atom_cache_empty_state() {
    let cache = AtomCache::new();

    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_atom_cache_not_empty_after_insert() {
    let mut cache = AtomCache::new();

    cache.insert(0, Atom::new("test"));

    assert!(!cache.is_empty());
    assert_eq!(cache.len(), 1);
}
