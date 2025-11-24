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

mod test_helpers;

use anyhow::Result;
use erltf::types::Atom;
use erltf::OwnedTerm;
use std::collections::BTreeMap;
use test_helpers::TestContext;

#[tokio::test]
async fn test_basic_rpc_test_function() -> Result<()> {
    let mut ctx = TestContext::new("basic").await?;

    let response = ctx.rpc_call("test_node", "test_function", vec![]).await?;
    let result = TestContext::unwrap_rex_response(response)?;

    if let OwnedTerm::Tuple(result) = result {
        assert_eq!(result.len(), 2);
        assert!(matches!(&result[0], OwnedTerm::Atom(a) if a == "ok"));

        if let OwnedTerm::List(chars) = &result[1] {
            let s: String = chars
                .iter()
                .filter_map(|t| {
                    if let OwnedTerm::Integer(i) = t {
                        Some(*i as u8 as char)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(s, "Hello from Erlang!");
        } else {
            panic!("Expected list (string), got {:?}", result[1]);
        }
    } else {
        panic!("Expected tuple, got {:?}", result);
    }

    Ok(())
}

#[tokio::test]
async fn test_simple_rpc_echo() -> Result<()> {
    let mut ctx = TestContext::new("echo").await?;

    let response = ctx
        .rpc_call(
            "test_node",
            "echo",
            vec![OwnedTerm::Atom(Atom::new("hello"))],
        )
        .await?;
    let result = TestContext::unwrap_rex_response(response)?;

    assert!(matches!(result, OwnedTerm::Atom(a) if a == "hello"));

    Ok(())
}

#[tokio::test]
async fn test_rpc_test_tuple() -> Result<()> {
    let mut ctx = TestContext::new("tuple").await?;

    let response = ctx.rpc_call("test_node", "test_tuple", vec![]).await?;
    let result = TestContext::unwrap_rex_response(response)?;

    if let OwnedTerm::Tuple(result) = result {
        assert_eq!(result.len(), 3);
        assert!(matches!(&result[0], OwnedTerm::Atom(a) if a == "ok"));
        assert!(matches!(&result[1], OwnedTerm::Atom(a) if a == "hello"));
        assert!(matches!(&result[2], OwnedTerm::Atom(a) if a == "world"));
    } else {
        panic!("Expected tuple, got {:?}", result);
    }

    Ok(())
}

#[tokio::test]
async fn test_rpc_add_integers() -> Result<()> {
    let mut ctx = TestContext::new("add").await?;

    let response = ctx
        .rpc_call(
            "test_node",
            "add",
            vec![OwnedTerm::Integer(10), OwnedTerm::Integer(32)],
        )
        .await?;
    let result = TestContext::unwrap_rex_response(response)?;

    assert!(matches!(result, OwnedTerm::Integer(42)));

    Ok(())
}

#[tokio::test]
async fn test_rpc_multiply_integers() -> Result<()> {
    let mut ctx = TestContext::new("multiply").await?;

    let response = ctx
        .rpc_call(
            "test_node",
            "multiply",
            vec![OwnedTerm::Integer(6), OwnedTerm::Integer(7)],
        )
        .await?;
    let result = TestContext::unwrap_rex_response(response)?;

    assert!(matches!(result, OwnedTerm::Integer(42)));

    Ok(())
}

#[tokio::test]
async fn test_rpc_list_operations() -> Result<()> {
    let mut ctx = TestContext::new("list").await?;

    let test_list = vec![
        OwnedTerm::Integer(1),
        OwnedTerm::Integer(2),
        OwnedTerm::Integer(3),
    ];

    let response = ctx
        .rpc_call(
            "test_node",
            "list_length",
            vec![OwnedTerm::List(test_list.clone())],
        )
        .await?;
    let result = TestContext::unwrap_rex_response(response)?;

    assert!(matches!(result, OwnedTerm::Integer(3)));

    let response = ctx
        .rpc_call(
            "test_node",
            "reverse_list",
            vec![OwnedTerm::List(test_list)],
        )
        .await?;
    let result = TestContext::unwrap_rex_response(response)?;

    if let OwnedTerm::List(reversed) = result {
        assert_eq!(reversed.len(), 3);
        assert!(matches!(reversed[0], OwnedTerm::Integer(3)));
        assert!(matches!(reversed[1], OwnedTerm::Integer(2)));
        assert!(matches!(reversed[2], OwnedTerm::Integer(1)));
    } else {
        panic!("Expected list, got {:?}", result);
    }

    Ok(())
}

#[tokio::test]
async fn test_rpc_make_list() -> Result<()> {
    let mut ctx = TestContext::new("make_list").await?;

    let response = ctx
        .rpc_call("test_node", "make_list", vec![OwnedTerm::Integer(5)])
        .await?;
    let result = TestContext::unwrap_rex_response(response)?;

    if let OwnedTerm::List(list) = result {
        assert_eq!(list.len(), 5);
        for (i, elem) in list.iter().enumerate() {
            assert!(matches!(elem, OwnedTerm::Integer(n) if *n == (i as i64 + 1)));
        }
    } else {
        panic!("Expected list, got {:?}", result);
    }

    Ok(())
}

#[tokio::test]
async fn test_rpc_get_node_name() -> Result<()> {
    let mut ctx = TestContext::new("node_name").await?;

    let response = ctx.rpc_call("test_node", "get_node_name", vec![]).await?;
    let _result = TestContext::unwrap_rex_response(response)?;

    Ok(())
}

#[tokio::test]
async fn test_rpc_error_handling() -> Result<()> {
    let mut ctx = TestContext::new("error").await?;

    let response = ctx.rpc_call("test_node", "return_error", vec![]).await?;
    let result = TestContext::unwrap_rex_response(response)?;

    if let OwnedTerm::Tuple(error_tuple) = result {
        assert_eq!(error_tuple.len(), 2);
        assert!(matches!(&error_tuple[0], OwnedTerm::Atom(a) if a == "error"));
        assert!(matches!(&error_tuple[1], OwnedTerm::Atom(a) if a == "intentional_error"));
    } else {
        panic!("Expected error tuple, got {:?}", result);
    }

    Ok(())
}

#[tokio::test]
async fn test_rpc_atom_to_string() -> Result<()> {
    let mut ctx = TestContext::new("atom_to_str").await?;

    let response = ctx
        .rpc_call(
            "test_node",
            "atom_to_string",
            vec![OwnedTerm::Atom(Atom::new("hello"))],
        )
        .await?;
    let result = TestContext::unwrap_rex_response(response)?;

    if let OwnedTerm::List(chars) = result {
        let s: String = chars
            .iter()
            .filter_map(|t| {
                if let OwnedTerm::Integer(i) = t {
                    Some(*i as u8 as char)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(s, "hello");
    } else {
        panic!("Expected list (string), got {:?}", result);
    }

    Ok(())
}

#[tokio::test]
async fn test_comprehensive_echo_data_structures() -> Result<()> {
    let mut ctx = TestContext::new("comprehensive").await?;

    let test_list = OwnedTerm::List(vec![
        OwnedTerm::Integer(1),
        OwnedTerm::Integer(2),
        OwnedTerm::Integer(3),
    ]);

    let mut map_data = BTreeMap::new();
    map_data.insert(OwnedTerm::Atom(Atom::new("key1")), OwnedTerm::Integer(100));
    map_data.insert(
        OwnedTerm::Atom(Atom::new("key2")),
        OwnedTerm::Atom(Atom::new("value")),
    );
    let test_map = OwnedTerm::Map(map_data);

    let test_binary = OwnedTerm::Binary(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);

    let test_string: Vec<OwnedTerm> = "hello"
        .chars()
        .map(|c| OwnedTerm::Integer(c as i64))
        .collect();
    let test_string_term = OwnedTerm::List(test_string);

    let test_improper = OwnedTerm::ImproperList {
        elements: vec![OwnedTerm::Integer(1), OwnedTerm::Integer(2)],
        tail: Box::new(OwnedTerm::Atom(Atom::new("tail"))),
    };

    let comprehensive_tuple = OwnedTerm::Tuple(vec![
        test_list.clone(),
        test_map.clone(),
        test_binary.clone(),
        test_string_term.clone(),
        test_improper.clone(),
    ]);

    let response = ctx
        .rpc_call("test_node", "echo", vec![comprehensive_tuple.clone()])
        .await?;
    let result = TestContext::unwrap_rex_response(response)?;

    assert_eq!(result, comprehensive_tuple);

    Ok(())
}

#[tokio::test]
async fn test_rpc_multiple_sequential_calls() -> Result<()> {
    let mut ctx = TestContext::new("sequential").await?;

    for i in 1..=5 {
        let response = ctx
            .rpc_call("test_node", "echo", vec![OwnedTerm::Integer(i)])
            .await?;
        let result = TestContext::unwrap_rex_response(response)?;

        assert!(matches!(result, OwnedTerm::Integer(n) if n == i));
    }

    Ok(())
}

#[tokio::test]
async fn test_rpc_concurrent_calls() -> Result<()> {
    let ctx = TestContext::new("concurrent").await?;

    let node = std::sync::Arc::new(tokio::sync::Mutex::new(ctx));

    let mut handles = vec![];
    for i in 1..=10 {
        let node_clone = node.clone();

        let handle = tokio::spawn(async move {
            let mut ctx_guard = node_clone.lock().await;
            let response = ctx_guard
                .rpc_call("test_node", "echo", vec![OwnedTerm::Integer(i)])
                .await?;
            TestContext::unwrap_rex_response(response)
        });
        handles.push((i, handle));
    }

    for (expected, handle) in handles {
        let result = handle.await??;
        assert!(matches!(result, OwnedTerm::Integer(n) if n == expected));
    }

    Ok(())
}
