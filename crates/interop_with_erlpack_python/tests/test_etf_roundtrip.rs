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

//! Integration tests for ETF round-trip between Rust (erltf) and Python (erlpack).
//!
//! These tests require Python 3 and erlpack to be installed:
//! `pip install erlpack`

use std::io::Write;
use std::process::{Command, Stdio};

fn python_available() -> bool {
    Command::new("python3").arg("--version").output().is_ok()
}

fn erlpack_installed() -> bool {
    Command::new("python3")
        .args(["-c", "import erlpack"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
#[ignore = "requires pip install erlpack"]
fn test_rust_encode_python_decode() {
    if !python_available() {
        eprintln!("Skipping: python3 not available");
        return;
    }
    if !erlpack_installed() {
        eprintln!("Skipping: erlpack not installed (pip install erlpack)");
        return;
    }

    let crate_dir = env!("CARGO_MANIFEST_DIR");

    // Build and run Rust encoder
    let rust_output = Command::new("cargo")
        .args(["run", "--bin", "encode_for_python"])
        .current_dir(crate_dir)
        .output()
        .expect("Failed to run Rust encoder");

    assert!(
        rust_output.status.success(),
        "Rust encoder failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );

    // Pipe to Python decoder
    let mut py_process = Command::new("python3")
        .arg("python/decode_from_rust.py")
        .current_dir(crate_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Python decoder");

    {
        let stdin = py_process.stdin.as_mut().unwrap();
        stdin.write_all(&rust_output.stdout).unwrap();
    }

    let py_output = py_process
        .wait_with_output()
        .expect("Failed to wait for Python");

    println!(
        "Python output: {}",
        String::from_utf8_lossy(&py_output.stdout)
    );
    if !py_output.stderr.is_empty() {
        eprintln!(
            "Python stderr: {}",
            String::from_utf8_lossy(&py_output.stderr)
        );
    }

    assert!(py_output.status.success(), "Python decoder failed");
}

#[test]
#[ignore = "requires pip install erlpack"]
fn test_python_encode_rust_decode() {
    if !python_available() {
        eprintln!("Skipping: python3 not available");
        return;
    }
    if !erlpack_installed() {
        eprintln!("Skipping: erlpack not installed (pip install erlpack)");
        return;
    }

    let crate_dir = env!("CARGO_MANIFEST_DIR");

    // Run Python encoder
    let py_output = Command::new("python3")
        .arg("python/encode_for_rust.py")
        .current_dir(crate_dir)
        .output()
        .expect("Failed to run Python encoder");

    assert!(
        py_output.status.success(),
        "Python encoder failed: {}",
        String::from_utf8_lossy(&py_output.stderr)
    );

    // Pipe to Rust decoder
    let mut rust_process = Command::new("cargo")
        .args(["run", "--bin", "decode_from_python"])
        .current_dir(crate_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Rust decoder");

    {
        let stdin = rust_process.stdin.as_mut().unwrap();
        stdin.write_all(&py_output.stdout).unwrap();
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
