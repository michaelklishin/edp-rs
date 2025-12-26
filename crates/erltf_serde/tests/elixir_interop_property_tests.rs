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

#![cfg(feature = "elixir-interop")]

use erltf_serde::{from_term, to_term};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    name: String,
    count: i64,
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct StructWithOption {
    name: String,
    value: Option<i64>,
}

fn arb_test_struct() -> impl Strategy<Value = TestStruct> {
    ("[a-z][a-z0-9_]{0,20}", any::<i64>(), any::<bool>()).prop_map(|(name, count, enabled)| {
        TestStruct {
            name,
            count,
            enabled,
        }
    })
}

proptest! {
    #[test]
    fn prop_roundtrip_option_i32(value in any::<Option<i32>>()) {
        let term = to_term(&value).unwrap();
        let result: Option<i32> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_roundtrip_option_i64(value in any::<Option<i64>>()) {
        let term = to_term(&value).unwrap();
        let result: Option<i64> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_roundtrip_option_f64(value in any::<Option<f64>>().prop_filter("finite floats only", |v| v.map_or(true, |f| f.is_finite()))) {
        let term = to_term(&value).unwrap();
        let result: Option<f64> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_roundtrip_option_bool(value in any::<Option<bool>>()) {
        let term = to_term(&value).unwrap();
        let result: Option<bool> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_roundtrip_option_string(value in proptest::option::of("[a-zA-Z0-9 ]{0,50}")) {
        let term = to_term(&value).unwrap();
        let result: Option<String> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_roundtrip_option_bytes(value in proptest::option::of(prop::collection::vec(any::<u8>(), 0..100))) {
        let term = to_term(&value).unwrap();
        let result: Option<Vec<u8>> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_roundtrip_vec_of_options(values in prop::collection::vec(any::<Option<i32>>(), 0..20)) {
        let term = to_term(&values).unwrap();
        let result: Vec<Option<i32>> = from_term(&term).unwrap();
        prop_assert_eq!(values, result);
    }

    #[test]
    fn prop_roundtrip_option_struct(value in proptest::option::of(arb_test_struct())) {
        let term = to_term(&value).unwrap();
        let result: Option<TestStruct> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_option_tuple_roundtrip(value in any::<Option<(i32, i32)>>()) {
        let term = to_term(&value).unwrap();
        let result: Option<(i32, i32)> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_option_vec_roundtrip(value in proptest::option::of(prop::collection::vec(any::<i32>(), 0..10))) {
        let term = to_term(&value).unwrap();
        let result: Option<Vec<i32>> = from_term(&term).unwrap();
        prop_assert_eq!(value, result);
    }

    #[test]
    fn prop_struct_with_optional_fields(
        name in "[a-z][a-z0-9_]{0,20}",
        opt_value in any::<Option<i64>>()
    ) {
        let val = StructWithOption {
            name,
            value: opt_value,
        };
        let term = to_term(&val).unwrap();
        let result: StructWithOption = from_term(&term).unwrap();
        prop_assert_eq!(val, result);
    }
}
