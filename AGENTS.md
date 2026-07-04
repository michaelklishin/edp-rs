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
as found in Erlang 26 and 27.

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

 * `nom` for binary parsing
 * `tokio`, the asynchronous runtime


## Rust Code Style

 * Prefer top-level `use` statements (imports) over fully qualified names: `Display` or `fmt::Display` with a `use` statement, not `std::fmt::Display`
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

## Comments, Writing Style and Voice

 * Only add important comments that express the non-obvious intent, both in tests and in the implementation
 * Keep the comments short
 * Pay attention to the grammar of your comments, including punctuation, full stops, articles, and so on
 * Do not add `()` to function names or references: use `KeyValueAccess.kv_get` and not `KeyValueAccess.kv_get()`

### Voice

Write like an engineer who values clarity and simplicity. This applies
to all prose: design docs, analyses, notes, and commit messages.

 * Plain and factual: state the why in one line, never narrate the what
 * Literal mechanism over metaphor: name the actual thing, not an image of it
 * Prefer the plainest word. No coined verbs, no jargon for its own sake
 * No flourish, no editorializing, no imagery. Real domain terms are fine
 * If a sentence needs a second clause to justify itself, it is probably too clever
 * Plain full sentences over compressed clever noun phrases: "a helper
   crate", not "a `tower`-shaped convenience"
 * State guarantees and behavior explicitly; do not leave them implied
   by jargon
 * Name tools and platforms precisely: `rustc` 1.92, edition 2024,
   crates.io, WebAssembly
 * No bold for emphasis; bold is for structural labels only, and sparingly
 * No "term — explanation" em-dash glosses: use ": " or parentheses
 * These vocabulary rules apply to identifiers too: test function names,
   helper modules, and fixture names use the same plain words as prose

### Writing and Markdown Style

 * Never add full stops to Markdown list items
 * Use "X and Y" in prose, not "X / Y" slash-shorthand. Exceptions: unit
   fractions (`bytes/sec`), single-concept abbreviations (`I/O`), and paths
   or code (`tests/unit/`, `src/lib.rs`)
 * Wrap code identifiers in backticks in prose: types like `Vec<T>`, traits
   like `Display`, functions like `Iterator::next`, modules, file names, and paths
 * Avoid robotic labels such as `**Thing / other:**`; write a plain sentence or a simple label
 * Match the existing conventions of the file and subdirectory you are
   editing: bullet character, heading depth, ID schemes, and table shape
   vary by project, and the local choice wins

## Git Instructions

 * Never add yourself to the list of commit co-authors
 * Never mention yourself in commit messages in any way (no "Generated by", no AI tool links, etc)

## Iterative Post-Implementation Review (IPIR)

Review the changes very carefully and holistically for correctness and safety,
opportunities to meaningfully simplify the implementation without losing
fidelity and effectiveness, the use of Rust idioms, the rich type system
patterns, meaningful test coverage, API usability and whether the changes are
worth adopting to begin with.

Look hard for ways to meaningfully improve both the tests and the implementation.

Perform 5 such iterations (holistic analysis runs).
