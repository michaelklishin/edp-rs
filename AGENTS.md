# Instructions for AI Agents

## Overview

This library implements the [Erlang Distribution Protocol](https://www.erlang.org/docs/26/apps/erts/erl_dist_protocol)
and targets Erlang 26+ plus very recent Rust (starting with `1.91.0`).


## Repository Layout

This is a Rust workspace managed by `cargo`. The repository layout is as follows:

 * `Cargo.toml`: the workspace manifest file
 * `crates/edp_client`: an Erlang Distribution Protocol client using Tokio
 * `crates/erltf`: an Erlang Term Format implementation
 * `crates/erltf_serde`: Serde glue for `erltf`
 * `crates/erltf_serde_derive`: `derive`-oriented Serde glue for `erltf`
 * `crates/edp_examples`: various examples that demonstrate the usage of this library suite

### The Erlang Term Format Crate, `erltf`

This library is the heart of the codebase: it implements the [Erlang Term Format](https://www.erlang.org/docs/26/apps/erts/erl_ext_dist.html)
implementation in Erlang 26 and 27.

### The Serde Glue, `erltf_serde`

A separate crate, `erltf_serde`, provides Serde glue for `erltf`.

### The Erlang Distribution Protocol Client Crate, `edp_client`

This crate implements an [Erlang Distribution Protocol](https://www.erlang.org/docs/26/apps/erts/erl_dist_protocol) client
using Tokio.


## Build System

 * To build the workspace, run `cargo build --all`
 * To run the tests, run `cargo nextest run --all`
 * To run benchmarks, use `cargo bench --package erltf`

## Target Rust Version

 * This tool targets cutting edge Rust (currently `1.91.0`)


## Key Dependencies

 * `nom` for parsing log files
 * `tokio` the asynchronous runtime


## Rust Code Style

 * Use top-level `use` statements (imports) to fully-qualified names, e.g. `Display` or `fmt::Display` with a `use` statement, to `std::fmt::Display`
 * Never use function-local `use` statements (imports)
 * Add tests to the modules under `tests`, never in the implementation files
 * At the end of each task, run `cargo fmt --all`
 * At the end of each task, run `cargo clippy --all` and fix any warnings it might emit

## Debugging

When troubleshooting Erlang Term Format or Erlang Distribution Protocol encoding, decoding and framing (fragmentation, fragmenting),
consider the following tools:

 * Use Wireshark (`tshark`) over `tcpdump` to capture network traffic, it does not require `sudo` permissions
 * When debugging examples, connect to Erlang or RabbitMQ nodes using `erl` and the remote shell module
 * Use Erlang tracing and traffic captures to compare example output to that of `erl` processes

## Tests

Tests should be descriptive and easy to read. Use property-based tests with `proptest` for edge cases
in addition to unit tests.

## Comments

 * Only add important comments that express the non-obvious intent, both in tests and in the implementation
 * Keep the comments short
 * Pay attention to the grammar of your comments, including punctuation, full stops, articles, and so on
 * Do not add `()` to function names or references: use `KeyValueAccess.kv_get` and not `KeyValueAccess.kv_get()`

## Git Instructions

 * Never add yourself to the list of commit co-authors

## Style Guide

 * Never add full stops to Markdown list items
