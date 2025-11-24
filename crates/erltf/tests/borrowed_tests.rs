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

use erltf::{BorrowedTerm, ParsingContext, PathSegment, decode, decode_borrowed, encode};
use std::borrow::Cow;

#[test]
fn test_borrowed_atom_zero_copy() {
    let data = encode(&erltf::OwnedTerm::atom("test")).unwrap();
    let borrowed = decode_borrowed(&data).unwrap();

    match borrowed {
        BorrowedTerm::Atom(s) => {
            assert_eq!(s, "test");
            assert!(matches!(s, Cow::Borrowed(_)), "Atom should be borrowed");
        }
        _ => panic!("Expected atom"),
    }
}

#[test]
fn test_borrowed_binary_zero_copy() {
    let data = encode(&erltf::OwnedTerm::binary(vec![1, 2, 3, 4, 5])).unwrap();
    let borrowed = decode_borrowed(&data).unwrap();

    match borrowed {
        BorrowedTerm::Binary(b) => {
            assert_eq!(b.as_ref(), &[1, 2, 3, 4, 5]);
            assert!(matches!(b, Cow::Borrowed(_)), "Binary should be borrowed");
        }
        _ => panic!("Expected binary"),
    }
}

#[test]
fn test_borrowed_to_owned_roundtrip() {
    let original = erltf::OwnedTerm::Tuple(vec![
        erltf::OwnedTerm::atom("test"),
        erltf::OwnedTerm::Integer(42),
        erltf::OwnedTerm::binary(vec![1, 2, 3]),
    ]);

    let data = encode(&original).unwrap();
    let borrowed = decode_borrowed(&data).unwrap();
    let owned = borrowed.to_owned();

    assert_eq!(original, owned);
}

#[test]
fn test_borrowed_is_borrowed_check() {
    let data = encode(&erltf::OwnedTerm::atom("test")).unwrap();
    let borrowed = decode_borrowed(&data).unwrap();
    assert!(borrowed.is_borrowed());

    let owned = borrowed.to_owned();
    let borrowed_from_owned = BorrowedTerm::from(&owned);
    assert!(borrowed_from_owned.is_borrowed());
}

#[test]
fn test_contextual_error_tuple() {
    let mut data = vec![131];
    data.push(104);
    data.push(2);
    data.push(97);
    data.push(42);

    let result = decode_borrowed(&data);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.context.display_path().contains("[1]"));
}

#[test]
fn test_contextual_error_list() {
    let mut data = vec![131];
    data.push(108);
    data.extend_from_slice(&[0, 0, 0, 2]);
    data.push(97);
    data.push(1);

    let result = decode_borrowed(&data);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.context.byte_offset > 0);
}

#[test]
fn test_contextual_error_map() {
    let term = erltf::OwnedTerm::Map(
        vec![
            (erltf::OwnedTerm::atom("key1"), erltf::OwnedTerm::Integer(1)),
            (erltf::OwnedTerm::atom("key2"), erltf::OwnedTerm::Integer(2)),
        ]
        .into_iter()
        .collect(),
    );

    let mut data = encode(&term).unwrap();
    data.truncate(data.len() - 1);

    let result = decode_borrowed(&data);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.context.byte_offset > 0);
}

#[test]
fn test_borrowed_ordering() {
    let data1 = encode(&erltf::OwnedTerm::atom("apple")).unwrap();
    let data2 = encode(&erltf::OwnedTerm::atom("banana")).unwrap();

    let term1 = decode_borrowed(&data1).unwrap();
    let term2 = decode_borrowed(&data2).unwrap();

    assert!(term1 < term2);
}

#[test]
fn test_borrowed_nested_structure_zero_copy() {
    let term = erltf::OwnedTerm::Tuple(vec![
        erltf::OwnedTerm::atom("ok"),
        erltf::OwnedTerm::List(vec![
            erltf::OwnedTerm::binary(vec![72, 101, 108, 108, 111]),
            erltf::OwnedTerm::atom("world"),
        ]),
    ]);

    let data = encode(&term).unwrap();
    let borrowed = decode_borrowed(&data).unwrap();

    assert!(borrowed.is_borrowed());

    match borrowed {
        BorrowedTerm::Tuple(ref elements) => {
            assert_eq!(elements.len(), 2);
            match &elements[0] {
                BorrowedTerm::Atom(s) => {
                    assert_eq!(s, "ok");
                    assert!(matches!(s, Cow::Borrowed(_)));
                }
                _ => panic!("Expected atom"),
            }
        }
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_borrowed_compare_with_owned() {
    let term = erltf::OwnedTerm::Tuple(vec![
        erltf::OwnedTerm::atom("test"),
        erltf::OwnedTerm::Integer(123),
    ]);

    let data = encode(&term).unwrap();
    let borrowed = decode_borrowed(&data).unwrap();
    let owned = decode(&data).unwrap();

    assert_eq!(borrowed.to_owned(), owned);
}

#[test]
fn test_parse_context_path_display() {
    let mut ctx = ParsingContext::new();
    ctx.push(PathSegment::TupleElement(0));
    ctx.push(PathSegment::ListElement(5));
    ctx.push(PathSegment::MapValue("key".to_string()));

    let path = ctx.display_path();
    assert_eq!(path, "root[0][5].key");
}
