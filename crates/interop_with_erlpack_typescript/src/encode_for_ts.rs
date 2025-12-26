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

//! Encodes various Erlang terms using erltf and writes them to stdout.
//! Each term is prefixed with a 4-byte big-endian length.

use std::collections::BTreeMap;
use std::io::Write;

use erltf::{OwnedTerm, encode};

fn main() -> anyhow::Result<()> {
    let test_cases: Vec<OwnedTerm> = vec![
        // Nil
        OwnedTerm::Nil,
        // Integers
        OwnedTerm::Integer(0),
        OwnedTerm::Integer(42),
        OwnedTerm::Integer(-100),
        OwnedTerm::Integer(255),
        OwnedTerm::Integer(256),
        OwnedTerm::Integer(65535),
        OwnedTerm::Integer(2147483647),
        OwnedTerm::Integer(-2147483648),
        // A float
        OwnedTerm::Float(1.23456),
        // Binaries (strings in erlpack)
        OwnedTerm::Binary(b"hello".to_vec()),
        OwnedTerm::Binary("unicode: \u{00e9}\u{00e8}\u{00ea}".as_bytes().to_vec()),
        // Lists
        OwnedTerm::List(vec![]),
        OwnedTerm::List(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::Integer(2),
            OwnedTerm::Integer(3),
        ]),
        OwnedTerm::List(vec![
            OwnedTerm::Binary(b"a".to_vec()),
            OwnedTerm::Binary(b"b".to_vec()),
            OwnedTerm::Binary(b"c".to_vec()),
        ]),
        // Maps
        {
            let mut map = BTreeMap::new();
            map.insert(
                OwnedTerm::Binary(b"key".to_vec()),
                OwnedTerm::Binary(b"value".to_vec()),
            );
            OwnedTerm::Map(map)
        },
        // A nested map
        {
            let mut deep = BTreeMap::new();
            deep.insert(OwnedTerm::Binary(b"value".to_vec()), OwnedTerm::Integer(42));
            let mut nested = BTreeMap::new();
            nested.insert(OwnedTerm::Binary(b"deep".to_vec()), OwnedTerm::Map(deep));
            let mut outer = BTreeMap::new();
            outer.insert(
                OwnedTerm::Binary(b"nested".to_vec()),
                OwnedTerm::Map(nested),
            );
            OwnedTerm::Map(outer)
        },
        // A nested list
        OwnedTerm::List(vec![
            OwnedTerm::Integer(1),
            OwnedTerm::List(vec![
                OwnedTerm::Integer(2),
                OwnedTerm::List(vec![OwnedTerm::Integer(3)]),
            ]),
        ]),
        // A mixed map
        {
            let mut map = BTreeMap::new();
            map.insert(
                OwnedTerm::Binary(b"list".to_vec()),
                OwnedTerm::List(vec![
                    OwnedTerm::Integer(1),
                    OwnedTerm::Integer(2),
                    OwnedTerm::Integer(3),
                ]),
            );
            map.insert(OwnedTerm::Binary(b"num".to_vec()), OwnedTerm::Integer(42));
            map.insert(
                OwnedTerm::Binary(b"str".to_vec()),
                OwnedTerm::Binary(b"test".to_vec()),
            );
            OwnedTerm::Map(map)
        },
    ];

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    for term in test_cases {
        let encoded = encode(&term)?;
        let len = encoded.len() as u32;
        handle.write_all(&len.to_be_bytes())?;
        handle.write_all(&encoded)?;
    }

    handle.flush()?;
    Ok(())
}
