// Copyright (C) 2025 Michael S. Klishin and Contributors
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

use edp_elixir_terms::{
    ArgumentError, AtomKeyMapBuilder, ElixirDate, ElixirDateTime, ElixirExceptionExt, ElixirMapSet,
    ElixirNaiveDateTime, ElixirRange, ElixirTime, GenServerTerms, KeyError, KeywordListBuilder,
    MatchError, RuntimeError, UndefinedFunctionError,
};
use erltf::{Atom, ExternalPid, OwnedTerm};
use std::collections::BTreeMap;

#[test]
fn keyword_list_basic() {
    let kw = KeywordListBuilder::new()
        .put("name", "Alice")
        .put("age", 30i64)
        .put("active", true)
        .build();

    assert!(kw.is_proplist());
    assert_eq!(kw.len(), 3);
}

#[test]
fn keyword_list_with_flags() {
    let kw = KeywordListBuilder::new()
        .put_flag("verbose")
        .put_flag("debug")
        .build();

    assert!(kw.is_proplist());
    assert_eq!(kw.len(), 2);
}

#[test]
fn atom_key_map_basic() {
    let map = AtomKeyMapBuilder::new()
        .insert("name", "Bob")
        .insert("count", 42i64)
        .build();

    assert!(map.is_map());
    assert_eq!(map.len(), 2);
}

#[test]
fn atom_key_map_as_struct() {
    let map = AtomKeyMapBuilder::new()
        .insert("name", "Charlie")
        .insert("email", "charlie@example.com")
        .build_struct("MyApp.User");

    assert!(map.is_elixir_struct());
    assert_eq!(map.elixir_struct_module(), Some("Elixir.MyApp.User"));
}

#[test]
fn conditional_insert() {
    let include_debug = false;
    let map = AtomKeyMapBuilder::new()
        .insert("name", "test")
        .insert_if(include_debug, "debug", true)
        .build();

    assert_eq!(map.len(), 1);
}

#[test]
fn optional_insert() {
    let maybe_value: Option<i64> = Some(42);
    let no_value: Option<i64> = None;

    let map = AtomKeyMapBuilder::new()
        .insert_some("present", maybe_value)
        .insert_some("absent", no_value)
        .build();

    assert_eq!(map.len(), 1);
}

#[test]
fn ascending_range() {
    let range = ElixirRange::ascending(1, 5);
    assert_eq!(range.len(), 5);
    assert!(range.contains(1));
    assert!(range.contains(3));
    assert!(range.contains(5));
    assert!(!range.contains(0));
    assert!(!range.contains(6));
}

#[test]
fn descending_range() {
    let range = ElixirRange::descending(5, 1);
    assert_eq!(range.len(), 5);
    assert!(range.contains(5));
    assert!(range.contains(3));
    assert!(range.contains(1));
}

#[test]
fn range_with_step() {
    let range = ElixirRange::new(1, 10, 2);
    assert_eq!(range.len(), 5);
    assert!(range.contains(1));
    assert!(range.contains(3));
    assert!(range.contains(5));
    assert!(!range.contains(2));
    assert!(!range.contains(4));
}

#[test]
fn empty_range() {
    let range = ElixirRange::new(10, 1, 1);
    assert!(range.is_empty());
    assert_eq!(range.len(), 0);
}

#[test]
fn range_iterator() {
    let range = ElixirRange::new(1, 5, 2);
    let values: Vec<i64> = range.into_iter().collect();
    assert_eq!(values, vec![1, 3, 5]);
}

#[test]
fn range_to_term_roundtrip() {
    let range = ElixirRange::new(1, 10, 1);
    let term: OwnedTerm = range.into();

    assert!(term.is_elixir_struct());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.Range"));

    let parsed = ElixirRange::from_term(&term).unwrap();
    assert_eq!(parsed.first, 1);
    assert_eq!(parsed.last, 10);
    assert_eq!(parsed.step, 1);
}

#[test]
fn descending_range_iterator() {
    let range = ElixirRange::descending(5, 1);
    let values: Vec<i64> = range.into_iter().collect();
    assert_eq!(values, vec![5, 4, 3, 2, 1]);
}

#[test]
fn range_iterator_with_step() {
    let range = ElixirRange::new(0, 10, 3);
    let values: Vec<i64> = range.into_iter().collect();
    assert_eq!(values, vec![0, 3, 6, 9]);
}

#[test]
fn range_iterator_size_hint() {
    let range = ElixirRange::ascending(1, 5);
    let mut iter = range.into_iter();

    assert_eq!(iter.len(), 5);
    iter.next();
    assert_eq!(iter.len(), 4);
    iter.next();
    iter.next();
    assert_eq!(iter.len(), 2);
}

#[test]
fn range_with_zero_step() {
    let range = ElixirRange::new(1, 10, 0);
    assert!(range.is_empty());
    assert_eq!(range.len(), 0);
}

#[test]
fn mapset_basic() {
    let mut set = ElixirMapSet::new();
    assert!(set.is_empty());

    set.insert(OwnedTerm::integer(1));
    set.insert(OwnedTerm::integer(2));
    set.insert(OwnedTerm::integer(3));

    assert_eq!(set.len(), 3);
    assert!(set.contains(&OwnedTerm::integer(2)));
    assert!(!set.contains(&OwnedTerm::integer(4)));
}

#[test]
fn mapset_rejects_duplicates() {
    let mut set = ElixirMapSet::new();
    set.insert(OwnedTerm::integer(1));
    set.insert(OwnedTerm::integer(1));
    set.insert(OwnedTerm::integer(1));

    assert_eq!(set.len(), 1);
}

#[test]
fn mapset_to_term() {
    let mut set = ElixirMapSet::new();
    set.insert(OwnedTerm::atom("a"));
    set.insert(OwnedTerm::atom("b"));

    let term: OwnedTerm = set.into();
    assert!(term.is_elixir_struct());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.MapSet"));
}

#[test]
fn mapset_from_term_roundtrip() {
    let mut set = ElixirMapSet::new();
    set.insert(OwnedTerm::integer(10));
    set.insert(OwnedTerm::integer(20));

    let term: OwnedTerm = set.into();
    let parsed = ElixirMapSet::from_term(&term).unwrap();

    assert_eq!(parsed.len(), 2);
    assert!(parsed.contains(&OwnedTerm::integer(10)));
    assert!(parsed.contains(&OwnedTerm::integer(20)));
}

#[test]
fn mapset_union() {
    let set1 = ElixirMapSet::from_values([1i64, 2, 3]);
    let set2 = ElixirMapSet::from_values([3i64, 4, 5]);

    let union = set1.union(&set2);
    assert_eq!(union.len(), 5);
}

#[test]
fn mapset_intersection() {
    let set1 = ElixirMapSet::from_values([1i64, 2, 3]);
    let set2 = ElixirMapSet::from_values([2i64, 3, 4]);

    let intersection = set1.intersection(&set2);
    assert_eq!(intersection.len(), 2);
}

#[test]
fn mapset_difference() {
    let set1 = ElixirMapSet::from_values([1i64, 2, 3]);
    let set2 = ElixirMapSet::from_values([2i64, 3, 4]);

    let diff = set1.difference(&set2);
    assert_eq!(diff.len(), 1);
    assert!(diff.contains(&OwnedTerm::integer(1)));
}

#[test]
fn mapset_symmetric_difference() {
    let set1 = ElixirMapSet::from_values([1i64, 2, 3]);
    let set2 = ElixirMapSet::from_values([2i64, 3, 4]);

    let sym_diff = set1.symmetric_difference(&set2);
    assert_eq!(sym_diff.len(), 2);
    assert!(sym_diff.contains(&OwnedTerm::integer(1)));
    assert!(sym_diff.contains(&OwnedTerm::integer(4)));
}

#[test]
fn mapset_subset_superset() {
    let set1 = ElixirMapSet::from_values([1i64, 2]);
    let set2 = ElixirMapSet::from_values([1i64, 2, 3]);

    assert!(set1.is_subset(&set2));
    assert!(!set2.is_subset(&set1));
    assert!(set2.is_superset(&set1));
    assert!(!set1.is_superset(&set2));
}

#[test]
fn mapset_disjoint() {
    let set1 = ElixirMapSet::from_values([1i64, 2]);
    let set2 = ElixirMapSet::from_values([3i64, 4]);
    let set3 = ElixirMapSet::from_values([2i64, 3]);

    assert!(set1.is_disjoint(&set2));
    assert!(!set1.is_disjoint(&set3));
}

#[test]
fn mapset_clear() {
    let mut set = ElixirMapSet::from_values([1i64, 2, 3]);
    assert_eq!(set.len(), 3);
    set.clear();
    assert!(set.is_empty());
}

#[test]
fn date_roundtrip() {
    let date = ElixirDate::new(2025, 12, 25);
    let term: OwnedTerm = date.into();

    assert!(term.is_elixir_struct());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.Date"));

    let parsed = ElixirDate::from_term(&term).unwrap();
    assert_eq!(parsed.year, 2025);
    assert_eq!(parsed.month, 12);
    assert_eq!(parsed.day, 25);
}

#[test]
fn time_roundtrip() {
    let time = ElixirTime::new(14, 30, 45, 123456, 6);
    let term: OwnedTerm = time.into();

    assert!(term.is_elixir_struct());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.Time"));

    let parsed = ElixirTime::from_term(&term).unwrap();
    assert_eq!(parsed.hour, 14);
    assert_eq!(parsed.minute, 30);
    assert_eq!(parsed.second, 45);
    assert_eq!(parsed.microsecond_value, 123456);
    assert_eq!(parsed.microsecond_precision, 6);
}

#[test]
fn datetime_utc_roundtrip() {
    let dt = ElixirDateTime::utc(2025, 12, 25, 14, 30, 0, 0, 0);
    let term: OwnedTerm = dt.into();

    assert!(term.is_elixir_struct());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.DateTime"));

    let parsed = ElixirDateTime::from_term(&term).unwrap();
    assert_eq!(parsed.year, 2025);
    assert_eq!(parsed.month, 12);
    assert_eq!(parsed.day, 25);
    assert_eq!(parsed.time_zone, "Etc/UTC");
}

#[test]
fn date_display() {
    let date = ElixirDate::new(2025, 1, 5);
    assert_eq!(date.to_string(), "~D[2025-01-05]");
}

#[test]
fn time_display() {
    let time = ElixirTime::hms(9, 5, 3);
    assert_eq!(time.to_string(), "~T[09:05:03]");

    let time_with_ms = ElixirTime::new(14, 30, 0, 123000, 3);
    assert_eq!(time_with_ms.to_string(), "~T[14:30:00.123]");
}

#[test]
fn naive_datetime_roundtrip() {
    let date = ElixirDate::new(2025, 12, 25);
    let time = ElixirTime::new(14, 30, 45, 0, 0);
    let ndt = ElixirNaiveDateTime::from_date_time(date, time);
    let term: OwnedTerm = ndt.into();

    assert!(term.is_elixir_struct());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.NaiveDateTime"));

    let parsed = ElixirNaiveDateTime::from_term(&term).unwrap();
    assert_eq!(parsed.year, 2025);
    assert_eq!(parsed.month, 12);
    assert_eq!(parsed.day, 25);
    assert_eq!(parsed.hour, 14);
    assert_eq!(parsed.minute, 30);
    assert_eq!(parsed.second, 45);
}

#[test]
fn argument_error_roundtrip() {
    let err = ArgumentError::new("invalid argument");
    let term = err.to_term();

    assert!(term.is_elixir_exception());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.ArgumentError"));

    let parsed = ArgumentError::from_term(&term).unwrap();
    assert_eq!(parsed.message, "invalid argument");
}

#[test]
fn runtime_error_roundtrip() {
    let err = RuntimeError::new("something went wrong");
    let term = err.to_term();

    assert!(term.is_elixir_exception());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.RuntimeError"));

    let parsed = RuntimeError::from_term(&term).unwrap();
    assert_eq!(parsed.message, "something went wrong");
}

#[test]
fn key_error_roundtrip() {
    let err = KeyError::new(
        OwnedTerm::atom("missing_key"),
        OwnedTerm::Map(BTreeMap::new()),
    );
    let term = err.to_term();

    assert!(term.is_elixir_exception());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.KeyError"));

    let parsed = KeyError::from_term(&term).unwrap();
    assert!(parsed.key.is_atom_with_name("missing_key"));
}

#[test]
fn key_error_with_message() {
    let err = KeyError::with_message(
        OwnedTerm::atom("name"),
        OwnedTerm::Map(BTreeMap::new()),
        "key :name not found",
    );
    let term = err.to_term();
    let parsed = KeyError::from_term(&term).unwrap();

    assert_eq!(parsed.message, Some("key :name not found".to_string()));
}

#[test]
fn match_error_roundtrip() {
    let err = MatchError::new(OwnedTerm::integer(42));
    let term = err.to_term();

    assert!(term.is_elixir_exception());
    assert_eq!(term.elixir_struct_module(), Some("Elixir.MatchError"));

    let parsed = MatchError::from_term(&term).unwrap();
    assert_eq!(parsed.term, OwnedTerm::integer(42));
}

#[test]
fn undefined_function_error_roundtrip() {
    let err = UndefinedFunctionError::new("MyModule", "my_function", 2);
    let term = err.to_term();

    assert!(term.is_elixir_exception());
    assert_eq!(
        term.elixir_struct_module(),
        Some("Elixir.UndefinedFunctionError")
    );

    let parsed = UndefinedFunctionError::from_term(&term).unwrap();
    assert_eq!(parsed.module, "MyModule");
    assert_eq!(parsed.function, "my_function");
    assert_eq!(parsed.arity, 2);
}

fn test_pid() -> ExternalPid {
    ExternalPid::new(Atom::new("test@localhost"), 0, 0, 0)
}

#[test]
fn gen_call_message() {
    let from_pid = OwnedTerm::Pid(test_pid());
    let from_ref = OwnedTerm::atom("ref");
    let request = OwnedTerm::atom("get_state");

    let msg = GenServerTerms::gen_call(from_pid, from_ref, request);
    assert!(GenServerTerms::is_gen_call(&msg));
}

#[test]
fn gen_cast_message() {
    let request = OwnedTerm::atom("do_something");
    let msg = GenServerTerms::gen_cast(request);
    assert!(GenServerTerms::is_gen_cast(&msg));
}

#[test]
fn genserver_reply() {
    let reply = GenServerTerms::reply(OwnedTerm::ok(), OwnedTerm::atom("state"));
    assert!(GenServerTerms::is_reply(&reply));
}

#[test]
fn genserver_noreply() {
    let noreply = GenServerTerms::noreply(OwnedTerm::atom("state"));
    assert!(GenServerTerms::is_noreply(&noreply));
}

#[test]
fn genserver_stop() {
    let stop = GenServerTerms::stop_normal(OwnedTerm::atom("state"));
    assert!(GenServerTerms::is_stop(&stop));
}

#[test]
fn init_responses() {
    let ok = GenServerTerms::init_ok(OwnedTerm::atom("initial_state"));
    assert!(ok.as_ok_tuple().is_some());

    let ignore = GenServerTerms::init_ignore();
    assert!(ignore.is_atom_with_name("ignore"));

    let stop = GenServerTerms::init_stop(OwnedTerm::atom("reason"));
    assert!(
        stop.as_2_tuple()
            .is_some_and(|(f, _)| f.is_atom_with_name("stop"))
    );
}

#[test]
fn parse_gen_call() {
    let from_pid = OwnedTerm::Pid(test_pid());
    let from_ref = OwnedTerm::atom("ref");
    let request = OwnedTerm::atom("ping");

    let msg = GenServerTerms::gen_call(from_pid, from_ref, request);
    let (from, req) = GenServerTerms::parse_gen_call(&msg).unwrap();

    assert!(from.as_2_tuple().is_some());
    assert!(req.is_atom_with_name("ping"));
}

#[test]
fn genserver_reply_with_timeout() {
    let reply = GenServerTerms::reply_with_timeout(OwnedTerm::ok(), OwnedTerm::atom("state"), 5000);
    assert!(GenServerTerms::is_reply(&reply));
    assert_eq!(reply.len(), 4);
}

#[test]
fn genserver_reply_hibernate() {
    let reply = GenServerTerms::reply_hibernate(OwnedTerm::ok(), OwnedTerm::atom("state"));
    assert!(GenServerTerms::is_reply(&reply));
}

#[test]
fn genserver_noreply_with_timeout() {
    let noreply = GenServerTerms::noreply_with_timeout(OwnedTerm::atom("state"), 1000);
    assert!(GenServerTerms::is_noreply(&noreply));
}

#[test]
fn genserver_stop_with_reply() {
    let stop = GenServerTerms::stop_with_reply(
        OwnedTerm::atom("normal"),
        OwnedTerm::ok(),
        OwnedTerm::atom("state"),
    );
    assert!(GenServerTerms::is_stop(&stop));
    assert_eq!(stop.len(), 4);
}

#[test]
fn parse_gen_cast() {
    let request = OwnedTerm::atom("async_request");
    let msg = GenServerTerms::gen_cast(request);

    let req = GenServerTerms::parse_gen_cast(&msg).unwrap();
    assert!(req.is_atom_with_name("async_request"));
}

#[test]
fn parse_gen_call_from() {
    let from_pid = OwnedTerm::Pid(test_pid());
    let from_ref = OwnedTerm::atom("ref123");
    let request = OwnedTerm::atom("get_status");

    let msg = GenServerTerms::gen_call(from_pid, from_ref, request);
    let (from, _) = GenServerTerms::parse_gen_call(&msg).unwrap();
    let (pid, ref_term) = GenServerTerms::parse_from(from).unwrap();

    assert!(pid.node.as_str().contains("test@localhost"));
    assert!(ref_term.is_atom_with_name("ref123"));
}

#[test]
fn date_validation_rejects_invalid() {
    assert!(ElixirDate::try_new(2025, 0, 15).is_none());
    assert!(ElixirDate::try_new(2025, 13, 15).is_none());
    assert!(ElixirDate::try_new(2025, 2, 30).is_none());
    assert!(ElixirDate::try_new(2025, 4, 31).is_none());
    assert!(ElixirDate::try_new(2025, 1, 0).is_none());
}

#[test]
fn date_validation_accepts_valid() {
    assert!(ElixirDate::try_new(2025, 1, 31).is_some());
    assert!(ElixirDate::try_new(2025, 2, 28).is_some());
    assert!(ElixirDate::try_new(2024, 2, 29).is_some()); // Leap year
    assert!(ElixirDate::try_new(2025, 2, 29).is_none()); // Not a leap year
}

#[test]
fn time_validation_rejects_invalid() {
    assert!(ElixirTime::try_new(24, 0, 0, 0, 0).is_none());
    assert!(ElixirTime::try_new(0, 60, 0, 0, 0).is_none());
    assert!(ElixirTime::try_new(0, 0, 60, 0, 0).is_none());
    assert!(ElixirTime::try_new(0, 0, 0, 1_000_000, 0).is_none());
    assert!(ElixirTime::try_new(0, 0, 0, 0, 7).is_none());
}

#[test]
fn time_validation_accepts_valid() {
    assert!(ElixirTime::try_new(23, 59, 59, 999_999, 6).is_some());
    assert!(ElixirTime::try_new(0, 0, 0, 0, 0).is_some());
}

mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn valid_dates_roundtrip(year in -9999i32..=9999i32, month in 1u8..=12u8, day in 1u8..=28u8) {
            let date = ElixirDate::try_new(year, month, day);
            prop_assert!(date.is_some());

            let date = date.unwrap();
            let term: OwnedTerm = date.into();
            let parsed = ElixirDate::from_term(&term);
            prop_assert!(parsed.is_some());

            let parsed = parsed.unwrap();
            prop_assert_eq!(parsed.year, year);
            prop_assert_eq!(parsed.month, month);
            prop_assert_eq!(parsed.day, day);
        }

        #[test]
        fn valid_times_roundtrip(hour in 0u8..=23u8, minute in 0u8..=59u8, second in 0u8..=59u8, microsecond in 0u32..=999_999u32) {
            let time = ElixirTime::try_new(hour, minute, second, microsecond, 6);
            prop_assert!(time.is_some());

            let time = time.unwrap();
            let term: OwnedTerm = time.into();
            let parsed = ElixirTime::from_term(&term);
            prop_assert!(parsed.is_some());

            let parsed = parsed.unwrap();
            prop_assert_eq!(parsed.hour, hour);
            prop_assert_eq!(parsed.minute, minute);
            prop_assert_eq!(parsed.second, second);
            prop_assert_eq!(parsed.microsecond_value, microsecond);
        }

        #[test]
        fn range_iterator_produces_correct_count(first in -100i64..=100i64, step in 1i64..=10i64, count in 0usize..=20usize) {
            let last = first + (step * count as i64) - step;
            if count > 0 {
                let range = ElixirRange::new(first, last, step);
                let values: Vec<i64> = range.into_iter().collect();
                prop_assert_eq!(values.len(), count);
            }
        }

        #[test]
        fn mapset_preserves_elements(values in prop::collection::vec(1i64..=1000i64, 0..50)) {
            let set = ElixirMapSet::from_values(values.clone());
            let term: OwnedTerm = set.into();
            let parsed = ElixirMapSet::from_term(&term);
            prop_assert!(parsed.is_some());

            let parsed = parsed.unwrap();
            for v in values {
                prop_assert!(parsed.contains(&OwnedTerm::integer(v)));
            }
        }
    }
}

#[test]
fn leap_year_century_rules() {
    assert!(!ElixirDate::is_leap_year(1900)); // Divisible by 100, not 400
    assert!(ElixirDate::is_leap_year(2000)); // Divisible by 400
    assert!(!ElixirDate::is_leap_year(2100)); // Divisible by 100, not 400
    assert!(ElixirDate::is_leap_year(2400)); // Divisible by 400
}

#[test]
fn naive_datetime_validation() {
    assert!(ElixirNaiveDateTime::try_new(2025, 2, 29, 12, 0, 0, 0, 0).is_none());
    assert!(ElixirNaiveDateTime::try_new(2024, 2, 29, 12, 0, 0, 0, 0).is_some());
    assert!(ElixirNaiveDateTime::try_new(2025, 1, 15, 25, 0, 0, 0, 0).is_none());
}

#[test]
fn datetime_utc_validation() {
    assert!(ElixirDateTime::try_utc(2025, 13, 1, 0, 0, 0, 0, 0).is_none());
    assert!(ElixirDateTime::try_utc(2025, 6, 15, 12, 30, 0, 0, 0).is_some());
}

#[test]
fn try_hms_validation() {
    assert!(ElixirTime::try_hms(23, 59, 59).is_some());
    assert!(ElixirTime::try_hms(24, 0, 0).is_none());
}

#[test]
fn descending_range_contains() {
    let range = ElixirRange::descending(10, 5);
    assert!(range.contains(10));
    assert!(range.contains(7));
    assert!(range.contains(5));
    assert!(!range.contains(4));
    assert!(!range.contains(11));
}

#[test]
fn range_contains_with_step() {
    let range = ElixirRange::new(0, 20, 5);
    assert!(range.contains(0));
    assert!(range.contains(5));
    assert!(range.contains(10));
    assert!(range.contains(15));
    assert!(range.contains(20));
    assert!(!range.contains(3));
    assert!(!range.contains(7));
}

#[test]
fn genserver_noreply_hibernate() {
    let noreply = GenServerTerms::noreply_hibernate(OwnedTerm::atom("state"));
    assert!(GenServerTerms::is_noreply(&noreply));
    assert_eq!(noreply.len(), 3);
}

#[test]
fn genserver_init_ok_with_timeout() {
    let init = GenServerTerms::init_ok_with_timeout(OwnedTerm::atom("state"), 5000);
    assert!(init.as_tuple().is_some());
    assert_eq!(init.len(), 3);
}

#[test]
fn genserver_init_ok_hibernate() {
    let init = GenServerTerms::init_ok_hibernate(OwnedTerm::atom("state"));
    assert!(init.as_tuple().is_some());
    assert_eq!(init.len(), 3);
}

#[test]
fn keyword_list_put_atom() {
    let kw = KeywordListBuilder::new()
        .put_atom("status", "active")
        .build();
    assert!(kw.is_proplist());
}

#[test]
fn keyword_list_put_term() {
    let kw = KeywordListBuilder::new()
        .put_term("data", OwnedTerm::List(vec![OwnedTerm::integer(1)]))
        .build();
    assert!(kw.is_proplist());
}

#[test]
fn keyword_list_extend() {
    let kw = KeywordListBuilder::new()
        .extend([("a", 1i64), ("b", 2i64)])
        .build();
    assert_eq!(kw.len(), 2);
}

#[test]
fn keyword_list_with_capacity() {
    let kw = KeywordListBuilder::with_capacity(10).put("x", 1i64).build();
    assert_eq!(kw.len(), 1);
}

#[test]
fn atom_key_map_insert_atom() {
    let map = AtomKeyMapBuilder::new()
        .insert_atom("status", "pending")
        .build();
    assert!(map.is_map());
}

#[test]
fn atom_key_map_insert_term() {
    let map = AtomKeyMapBuilder::new()
        .insert_term("list", OwnedTerm::List(vec![]))
        .build();
    assert!(map.is_map());
}

#[test]
fn atom_key_map_extend() {
    let map = AtomKeyMapBuilder::new()
        .extend([("x", 1i64), ("y", 2i64)])
        .build();
    assert_eq!(map.len(), 2);
}

#[test]
fn keyword_list_is_empty() {
    let builder = KeywordListBuilder::new();
    assert!(builder.is_empty());

    let builder = builder.put("key", 1i64);
    assert!(!builder.is_empty());
}

#[test]
fn keyword_list_put_if() {
    let kw = KeywordListBuilder::new()
        .put_if(true, "included", 1i64)
        .put_if(false, "excluded", 2i64)
        .build();
    assert_eq!(kw.len(), 1);
}

#[test]
fn atom_key_map_is_empty() {
    let builder = AtomKeyMapBuilder::new();
    assert!(builder.is_empty());

    let builder = builder.insert("key", 1i64);
    assert!(!builder.is_empty());
}

#[test]
fn mapset_remove() {
    let mut set = ElixirMapSet::new();
    set.insert(OwnedTerm::integer(1));
    set.insert(OwnedTerm::integer(2));
    assert_eq!(set.len(), 2);

    assert!(set.remove(&OwnedTerm::integer(1)));
    assert_eq!(set.len(), 1);
    assert!(!set.contains(&OwnedTerm::integer(1)));

    assert!(!set.remove(&OwnedTerm::integer(99)));
}

#[test]
fn mapset_iter() {
    let set = ElixirMapSet::from_values([1i64, 2, 3]);
    let count = set.iter().count();
    assert_eq!(count, 3);
}

#[test]
fn datetime_with_timezone() {
    let dt = ElixirDateTime::with_timezone(
        2025,
        6,
        15,
        10,
        30,
        0,
        0,
        0,
        "America/New_York",
        "EDT",
        -18000,
        3600,
    );
    assert_eq!(dt.time_zone, "America/New_York");
    assert_eq!(dt.zone_abbr, "EDT");
    assert_eq!(dt.utc_offset, -18000);
    assert_eq!(dt.std_offset, 3600);
}

#[test]
fn undefined_function_error_with_reason() {
    let err = UndefinedFunctionError::with_reason("Enum", "map", 2, "function not exported");
    let term = err.to_term();
    let parsed = UndefinedFunctionError::from_term(&term).unwrap();
    assert_eq!(parsed.reason, Some("function not exported".to_string()));
}

#[test]
fn single_element_range() {
    let range = ElixirRange::new(5, 5, 1);
    assert_eq!(range.len(), 1);
    assert!(range.contains(5));
    let values: Vec<i64> = range.into_iter().collect();
    assert_eq!(values, vec![5]);
}

#[test]
fn negative_step_range() {
    let range = ElixirRange::new(10, 0, -2);
    assert_eq!(range.len(), 6);
    let values: Vec<i64> = range.into_iter().collect();
    assert_eq!(values, vec![10, 8, 6, 4, 2, 0]);
}

#[test]
fn range_display() {
    let range = ElixirRange::ascending(1, 10);
    assert_eq!(range.to_string(), "1..10");

    let stepped = ElixirRange::new(1, 10, 2);
    assert_eq!(stepped.to_string(), "1..10//2");
}

#[test]
fn keyword_list_put_some() {
    let present: Option<i64> = Some(42);
    let absent: Option<i64> = None;

    let kw = KeywordListBuilder::new()
        .put_some("present", present)
        .put_some("absent", absent)
        .build();

    assert_eq!(kw.len(), 1);
}

#[test]
fn genserver_stop_shutdown() {
    let stop = GenServerTerms::stop_shutdown(OwnedTerm::atom("state"));
    assert!(GenServerTerms::is_stop(&stop));
}

#[test]
fn mapset_from_iterator() {
    let terms: Vec<OwnedTerm> = vec![
        OwnedTerm::integer(1),
        OwnedTerm::integer(2),
        OwnedTerm::integer(3),
    ];
    let set: ElixirMapSet = terms.into_iter().collect();
    assert_eq!(set.len(), 3);
}

#[test]
fn mapset_into_iter_owned() {
    let set = ElixirMapSet::from_values([1i64, 2, 3]);
    let collected: Vec<OwnedTerm> = set.into_iter().collect();
    assert_eq!(collected.len(), 3);
}

#[test]
fn naive_datetime_display() {
    let ndt = ElixirNaiveDateTime::new(2025, 12, 25, 14, 30, 0, 0, 0);
    assert_eq!(ndt.to_string(), "~N[2025-12-25 14:30:00]");

    let ndt_with_ms = ElixirNaiveDateTime::new(2025, 6, 15, 10, 0, 0, 123000, 3);
    assert_eq!(ndt_with_ms.to_string(), "~N[2025-06-15 10:00:00.123]");
}

#[test]
fn datetime_display() {
    let dt = ElixirDateTime::utc(2025, 12, 25, 14, 30, 0, 0, 0);
    assert_eq!(dt.to_string(), "~U[2025-12-25 14:30:00Z]");

    let dt_tz = ElixirDateTime::with_timezone(
        2025,
        6,
        15,
        10,
        30,
        0,
        0,
        0,
        "America/New_York",
        "EDT",
        -18000,
        3600,
    );
    assert_eq!(dt_tz.to_string(), "~U[2025-06-15 10:30:00EDT]");
}

#[test]
fn mapset_borrow_iter() {
    let set = ElixirMapSet::from_values([1i64, 2, 3]);
    let mut count = 0;
    for _item in &set {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn exception_from_trait() {
    let arg_err = ArgumentError::new("bad argument");
    let term: OwnedTerm = arg_err.into();
    assert!(term.is_elixir_exception());

    let rt_err = RuntimeError::new("runtime error");
    let term: OwnedTerm = rt_err.into();
    assert!(term.is_elixir_exception());

    let match_err = MatchError::new(OwnedTerm::integer(42));
    let term: OwnedTerm = match_err.into();
    assert!(term.is_elixir_exception());
}

#[test]
fn range_with_negative_values() {
    let range = ElixirRange::new(-10, -5, 1);
    assert_eq!(range.len(), 6);
    assert!(range.contains(-10));
    assert!(range.contains(-7));
    assert!(range.contains(-5));
    assert!(!range.contains(-4));

    let values: Vec<i64> = range.into_iter().collect();
    assert_eq!(values, vec![-10, -9, -8, -7, -6, -5]);
}

#[test]
fn from_term_rejects_wrong_struct_type() {
    let date_term: OwnedTerm = ElixirDate::new(2025, 1, 1).into();
    assert!(ElixirTime::from_term(&date_term).is_none());
    assert!(ElixirRange::from_term(&date_term).is_none());
    assert!(ElixirMapSet::from_term(&date_term).is_none());
}

#[test]
fn from_term_rejects_non_struct() {
    let plain_map = OwnedTerm::Map(BTreeMap::new());
    assert!(ElixirDate::from_term(&plain_map).is_none());

    let integer = OwnedTerm::integer(42);
    assert!(ElixirRange::from_term(&integer).is_none());
}

#[test]
fn datetime_display_with_microseconds() {
    let dt = ElixirDateTime::utc(2025, 6, 15, 10, 30, 45, 123456, 6);
    assert_eq!(dt.to_string(), "~U[2025-06-15 10:30:45.123456Z]");
}

#[test]
fn mapset_empty_operations() {
    let empty = ElixirMapSet::new();
    let set = ElixirMapSet::from_values([1i64, 2, 3]);

    assert_eq!(empty.union(&set).len(), 3);
    assert_eq!(set.union(&empty).len(), 3);
    assert!(empty.intersection(&set).is_empty());
    assert!(set.difference(&set).is_empty());
}

#[test]
fn key_error_from_trait() {
    let err = KeyError::new(OwnedTerm::atom("key"), OwnedTerm::Map(BTreeMap::new()));
    let term: OwnedTerm = err.into();
    assert!(term.is_elixir_exception());
}

#[test]
fn undefined_function_error_from_trait() {
    let err = UndefinedFunctionError::new("Module", "func", 1);
    let term: OwnedTerm = err.into();
    assert!(term.is_elixir_exception());
}

#[test]
fn naive_datetime_from_date_time() {
    let date = ElixirDate::new(2025, 6, 15);
    let time = ElixirTime::new(10, 30, 45, 500000, 6);
    let ndt = ElixirNaiveDateTime::from_date_time(date, time);

    assert_eq!(ndt.year, 2025);
    assert_eq!(ndt.month, 6);
    assert_eq!(ndt.day, 15);
    assert_eq!(ndt.hour, 10);
    assert_eq!(ndt.minute, 30);
    assert_eq!(ndt.second, 45);
    assert_eq!(ndt.microsecond_value, 500000);
}

#[test]
fn genserver_predicates_negative_cases() {
    let not_gen_call = OwnedTerm::atom("hello");
    assert!(!GenServerTerms::is_gen_call(&not_gen_call));
    assert!(!GenServerTerms::is_gen_cast(&not_gen_call));
    assert!(!GenServerTerms::is_reply(&not_gen_call));
    assert!(!GenServerTerms::is_noreply(&not_gen_call));
    assert!(!GenServerTerms::is_stop(&not_gen_call));
}

#[test]
fn genserver_parse_negative_cases() {
    let not_gen_call = OwnedTerm::atom("hello");
    assert!(GenServerTerms::parse_gen_call(&not_gen_call).is_none());
    assert!(GenServerTerms::parse_gen_cast(&not_gen_call).is_none());
    assert!(GenServerTerms::parse_from(&not_gen_call).is_none());
}

#[test]
fn empty_range_iteration() {
    let range = ElixirRange::new(10, 1, 1);
    let values: Vec<i64> = range.into_iter().collect();
    assert!(values.is_empty());
}

#[test]
fn descending_range_display() {
    let range = ElixirRange::descending(10, 1);
    assert_eq!(range.to_string(), "10..1//-1");
}

#[test]
fn exception_module_names() {
    assert_eq!(ArgumentError::module_name(), "Elixir.ArgumentError");
    assert_eq!(RuntimeError::module_name(), "Elixir.RuntimeError");
    assert_eq!(KeyError::module_name(), "Elixir.KeyError");
    assert_eq!(MatchError::module_name(), "Elixir.MatchError");
    assert_eq!(
        UndefinedFunctionError::module_name(),
        "Elixir.UndefinedFunctionError"
    );
}

#[test]
fn datetime_from_term_rejects_wrong_type() {
    let naive_dt: OwnedTerm = ElixirNaiveDateTime::new(2025, 1, 1, 0, 0, 0, 0, 0).into();
    assert!(ElixirDateTime::from_term(&naive_dt).is_none());
}

#[test]
fn naive_datetime_from_term_rejects_wrong_type() {
    let dt: OwnedTerm = ElixirDateTime::utc(2025, 1, 1, 0, 0, 0, 0, 0).into();
    assert!(ElixirNaiveDateTime::from_term(&dt).is_none());
}

#[test]
fn exception_from_term_rejects_wrong_type() {
    let arg_err: OwnedTerm = ArgumentError::new("test").into();
    assert!(RuntimeError::from_term(&arg_err).is_none());
    assert!(KeyError::from_term(&arg_err).is_none());
    assert!(MatchError::from_term(&arg_err).is_none());
    assert!(UndefinedFunctionError::from_term(&arg_err).is_none());
}

#[test]
fn time_display_with_varying_precision() {
    let time_p1 = ElixirTime::new(14, 30, 0, 100000, 1);
    assert_eq!(time_p1.to_string(), "~T[14:30:00.1]");

    let time_p3 = ElixirTime::new(14, 30, 0, 123000, 3);
    assert_eq!(time_p3.to_string(), "~T[14:30:00.123]");

    let time_p6 = ElixirTime::new(14, 30, 0, 123456, 6);
    assert_eq!(time_p6.to_string(), "~T[14:30:00.123456]");
}

#[test]
fn datetime_with_timezone_roundtrip() {
    let dt = ElixirDateTime::with_timezone(
        2025,
        6,
        15,
        10,
        30,
        45,
        123456,
        6,
        "America/New_York",
        "EDT",
        -18000,
        3600,
    );
    let term: OwnedTerm = dt.into();
    let parsed = ElixirDateTime::from_term(&term).unwrap();

    assert_eq!(parsed.year, 2025);
    assert_eq!(parsed.month, 6);
    assert_eq!(parsed.day, 15);
    assert_eq!(parsed.hour, 10);
    assert_eq!(parsed.minute, 30);
    assert_eq!(parsed.second, 45);
    assert_eq!(parsed.microsecond_value, 123456);
    assert_eq!(parsed.time_zone, "America/New_York");
    assert_eq!(parsed.zone_abbr, "EDT");
    assert_eq!(parsed.utc_offset, -18000);
    assert_eq!(parsed.std_offset, 3600);
}

#[test]
fn date_negative_year() {
    let date = ElixirDate::new(-500, 3, 15);
    let term: OwnedTerm = date.into();
    let parsed = ElixirDate::from_term(&term).unwrap();

    assert_eq!(parsed.year, -500);
    assert_eq!(date.to_string(), "~D[-500-03-15]");
}

#[test]
fn date_year_zero() {
    let date = ElixirDate::new(0, 1, 1);
    assert_eq!(date.to_string(), "~D[0000-01-01]");
}

#[test]
fn range_large_values() {
    let range = ElixirRange::new(0, 1_000_000, 1000);
    assert_eq!(range.len(), 1001);
    assert!(range.contains(0));
    assert!(range.contains(1000));
    assert!(range.contains(1_000_000));
    assert!(!range.contains(500));
}

#[test]
fn genserver_stop_custom_reason() {
    let stop = GenServerTerms::stop(OwnedTerm::atom("killed"), OwnedTerm::atom("state"));
    assert!(GenServerTerms::is_stop(&stop));
    assert_eq!(stop.len(), 3);
}

#[test]
fn range_iterator_at_boundary() {
    let range = ElixirRange::new(i64::MAX - 2, i64::MAX, 1);
    let values: Vec<i64> = range.into_iter().collect();
    assert_eq!(values, vec![i64::MAX - 2, i64::MAX - 1, i64::MAX]);
}

#[test]
fn range_iterator_at_min_boundary() {
    let range = ElixirRange::new(i64::MIN + 2, i64::MIN, -1);
    let values: Vec<i64> = range.into_iter().collect();
    assert_eq!(values, vec![i64::MIN + 2, i64::MIN + 1, i64::MIN]);
}

#[test]
fn naive_datetime_to_date_time() {
    let ndt = ElixirNaiveDateTime::new(2025, 6, 15, 14, 30, 45, 123456, 6);
    let date = ndt.to_date();
    let time = ndt.to_time();

    assert_eq!(date.year, 2025);
    assert_eq!(date.month, 6);
    assert_eq!(date.day, 15);
    assert_eq!(time.hour, 14);
    assert_eq!(time.minute, 30);
    assert_eq!(time.second, 45);
    assert_eq!(time.microsecond_value, 123456);
}

#[test]
fn datetime_to_naive() {
    let dt = ElixirDateTime::utc(2025, 6, 15, 14, 30, 45, 123456, 6);
    let naive = dt.to_naive();

    assert_eq!(naive.year, 2025);
    assert_eq!(naive.month, 6);
    assert_eq!(naive.day, 15);
    assert_eq!(naive.hour, 14);
    assert_eq!(naive.microsecond_value, 123456);
}
