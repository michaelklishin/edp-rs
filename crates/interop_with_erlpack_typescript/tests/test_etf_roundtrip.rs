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

//! Integration tests for ETF round-trip between Rust (erltf) and TypeScript (erlpack).
//!
//! These tests require Node.js and npm to be installed, and `npm install` to have
//! been run in this crate's directory.

use std::io::Write;
use std::process::{Command, Stdio};

fn npm_installed() -> bool {
    Command::new("npm").arg("--version").output().is_ok()
}

fn node_modules_exist() -> bool {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("node_modules")
        .exists()
}

#[test]
#[ignore = "requires npm install"]
fn test_rust_encode_typescript_decode() {
    if !npm_installed() {
        eprintln!("Skipping: npm not installed");
        return;
    }
    if !node_modules_exist() {
        eprintln!("Skipping: run 'npm install' first");
        return;
    }

    let crate_dir = env!("CARGO_MANIFEST_DIR");

    // Build and run Rust encoder
    let rust_output = Command::new("cargo")
        .args(["run", "--bin", "encode_for_typescript"])
        .current_dir(crate_dir)
        .output()
        .expect("Failed to run Rust encoder");

    assert!(
        rust_output.status.success(),
        "Rust encoder failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );

    // Pipe to TypeScript decoder
    let mut ts_process = Command::new("npx")
        .args(["ts-node", "--esm", "typescript/decode_from_rust.ts"])
        .current_dir(crate_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn TypeScript decoder");

    {
        let stdin = ts_process.stdin.as_mut().unwrap();
        stdin.write_all(&rust_output.stdout).unwrap();
    }

    let ts_output = ts_process
        .wait_with_output()
        .expect("Failed to wait for TypeScript");

    println!(
        "TypeScript output: {}",
        String::from_utf8_lossy(&ts_output.stdout)
    );
    if !ts_output.stderr.is_empty() {
        eprintln!(
            "TypeScript stderr: {}",
            String::from_utf8_lossy(&ts_output.stderr)
        );
    }

    assert!(ts_output.status.success(), "TypeScript decoder failed");
}

#[test]
#[ignore = "requires npm install"]
fn test_typescript_encode_rust_decode() {
    if !npm_installed() {
        eprintln!("Skipping: npm not installed");
        return;
    }
    if !node_modules_exist() {
        eprintln!("Skipping: run 'npm install' first");
        return;
    }

    let crate_dir = env!("CARGO_MANIFEST_DIR");

    // Run TypeScript encoder
    let ts_output = Command::new("npx")
        .args(["ts-node", "--esm", "typescript/encode_for_rust.ts"])
        .current_dir(crate_dir)
        .output()
        .expect("Failed to run TypeScript encoder");

    assert!(
        ts_output.status.success(),
        "TypeScript encoder failed: {}",
        String::from_utf8_lossy(&ts_output.stderr)
    );

    // Pipe to Rust decoder
    let mut rust_process = Command::new("cargo")
        .args(["run", "--bin", "decode_from_typescript"])
        .current_dir(crate_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Rust decoder");

    {
        let stdin = rust_process.stdin.as_mut().unwrap();
        stdin.write_all(&ts_output.stdout).unwrap();
    }

    let rust_output = rust_process
        .wait_with_output()
        .expect("Failed to wait for Rust");

    println!(
        "Rust output: {}",
        String::from_utf8_lossy(&rust_output.stdout)
    );
    if !rust_output.stderr.is_empty() {
        eprintln!(
            "Rust stderr: {}",
            String::from_utf8_lossy(&rust_output.stderr)
        );
    }

    assert!(rust_output.status.success(), "Rust decoder failed");
}
