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

use edp_client::epmd_client::{NodeType, Protocol};

#[test]
fn test_node_type_values() {
    assert_eq!(NodeType::Normal as u8, 77);
    assert_eq!(NodeType::Hidden as u8, 72);
    assert_eq!(NodeType::R3Hidden as u8, 104);
}

#[test]
fn test_protocol_tcp() {
    assert_eq!(Protocol::Tcp as u8, 0);
}

#[test]
fn test_node_type_discriminants() {
    assert_ne!(NodeType::Normal as u8, NodeType::Hidden as u8);
    assert_ne!(NodeType::Normal as u8, NodeType::R3Hidden as u8);
    assert_ne!(NodeType::Hidden as u8, NodeType::R3Hidden as u8);
}
