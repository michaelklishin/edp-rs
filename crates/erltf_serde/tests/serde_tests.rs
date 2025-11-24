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

use erltf_serde::{from_bytes, from_term, to_bytes, to_term};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//
// Primitive Types
//

#[test]
fn test_serialize_deserialize_bool() {
    let val = true;
    let bytes = to_bytes(&val).unwrap();
    let result: bool = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);

    let val = false;
    let bytes = to_bytes(&val).unwrap();
    let result: bool = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_serialize_deserialize_integers() {
    let val: i32 = 42;
    let bytes = to_bytes(&val).unwrap();
    let result: i32 = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);

    let val: i64 = -12345;
    let bytes = to_bytes(&val).unwrap();
    let result: i64 = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_u64_max_roundtrip() {
    let val: u64 = u64::MAX;
    let bytes = to_bytes(&val).unwrap();
    let result: u64 = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_large_u64_as_bigint() {
    let val: u64 = i64::MAX as u64 + 1000;
    let bytes = to_bytes(&val).unwrap();
    let result: u64 = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_serialize_deserialize_float() {
    let val: f64 = 2.5;
    let bytes = to_bytes(&val).unwrap();
    let result: f64 = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_serialize_deserialize_string() {
    let val = "hello world";
    let bytes = to_bytes(&val).unwrap();
    let result: String = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

//
// Collections
//

#[test]
fn test_serialize_deserialize_bytes() {
    let val: Vec<u8> = vec![1, 2, 3, 4, 5];
    let bytes = to_bytes(&val).unwrap();
    let result: Vec<u8> = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_serialize_deserialize_option() {
    let val: Option<i32> = Some(42);
    let bytes = to_bytes(&val).unwrap();
    let result: Option<i32> = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);

    let val: Option<i32> = None;
    let bytes = to_bytes(&val).unwrap();
    let result: Option<i32> = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_serialize_deserialize_vec() {
    let val = vec![1, 2, 3, 4, 5];
    let bytes = to_bytes(&val).unwrap();
    let result: Vec<i32> = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_serialize_deserialize_tuple() {
    let val = (42, "hello", 3.5);
    let bytes = to_bytes(&val).unwrap();
    let result: (i32, String, f64) = from_bytes(&bytes).unwrap();
    assert_eq!(val.0, result.0);
    assert_eq!(val.1, result.1);
    assert_eq!(val.2, result.2);
}

#[test]
fn test_serialize_deserialize_map() {
    let mut map = HashMap::new();
    map.insert("key1".to_string(), 100);
    map.insert("key2".to_string(), 200);

    let bytes = to_bytes(&map).unwrap();
    let result: HashMap<String, i32> = from_bytes(&bytes).unwrap();
    assert_eq!(map, result);
}

//
// Structs
//

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SimpleStruct {
    name: String,
    age: i32,
    active: bool,
}

#[test]
fn test_serialize_deserialize_struct() {
    let val = SimpleStruct {
        name: "Alice".to_string(),
        age: 30,
        active: true,
    };

    let bytes = to_bytes(&val).unwrap();
    let result: SimpleStruct = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NestedStruct {
    id: i32,
    data: SimpleStruct,
    tags: Vec<String>,
}

#[test]
fn test_serialize_deserialize_nested_struct() {
    let val = NestedStruct {
        id: 1,
        data: SimpleStruct {
            name: "Bob".to_string(),
            age: 25,
            active: false,
        },
        tags: vec!["tag1".to_string(), "tag2".to_string()],
    };

    let bytes = to_bytes(&val).unwrap();
    let result: NestedStruct = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

//
// Enums
//

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum SimpleEnum {
    Variant1,
    Variant2,
}

#[test]
fn test_serialize_deserialize_unit_enum() {
    let val = SimpleEnum::Variant1;
    let bytes = to_bytes(&val).unwrap();
    let result: SimpleEnum = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum ComplexEnum {
    Unit,
    Newtype(i32),
    Tuple(i32, String),
    Struct { x: i32, y: String },
}

#[test]
fn test_serialize_deserialize_complex_enum() {
    let val = ComplexEnum::Unit;
    let bytes = to_bytes(&val).unwrap();
    let result: ComplexEnum = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);

    let val = ComplexEnum::Newtype(42);
    let bytes = to_bytes(&val).unwrap();
    let result: ComplexEnum = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);

    let val = ComplexEnum::Tuple(100, "test".to_string());
    let bytes = to_bytes(&val).unwrap();
    let result: ComplexEnum = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);

    let val = ComplexEnum::Struct {
        x: 99,
        y: "data".to_string(),
    };
    let bytes = to_bytes(&val).unwrap();
    let result: ComplexEnum = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

//
// Term Conversion
//

#[test]
fn test_to_term_from_term() {
    let val = SimpleStruct {
        name: "Charlie".to_string(),
        age: 35,
        active: true,
    };

    let term = to_term(&val).unwrap();
    let result: SimpleStruct = from_term(&term).unwrap();
    assert_eq!(val, result);
}

//
// Edge Cases
//

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct WithOption {
    required: String,
    optional: Option<i32>,
}

#[test]
fn test_serialize_deserialize_struct_with_option() {
    let val = WithOption {
        required: "test".to_string(),
        optional: Some(42),
    };
    let bytes = to_bytes(&val).unwrap();
    let result: WithOption = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);

    let val = WithOption {
        required: "test".to_string(),
        optional: None,
    };
    let bytes = to_bytes(&val).unwrap();
    let result: WithOption = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_serialize_deserialize_empty_vec() {
    let val: Vec<i32> = vec![];
    let bytes = to_bytes(&val).unwrap();
    let result: Vec<i32> = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}

#[test]
fn test_serialize_deserialize_empty_map() {
    let val: HashMap<String, i32> = HashMap::new();
    let bytes = to_bytes(&val).unwrap();
    let result: HashMap<String, i32> = from_bytes(&bytes).unwrap();
    assert_eq!(val, result);
}
