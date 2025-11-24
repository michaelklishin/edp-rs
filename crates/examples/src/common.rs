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

//! Common utilities shared across examples.

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Reads the Erlang cookie from ~/.erlang.cookie
pub fn get_erlang_cookie() -> Result<String> {
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let cookie_path = PathBuf::from(home).join(".erlang.cookie");
    let cookie = fs::read_to_string(&cookie_path)
        .with_context(|| format!("Failed to read Erlang cookie from {:?}", cookie_path))?;
    Ok(cookie.trim().to_string())
}

/// Gets the system hostname, optionally using short or long names
pub fn get_hostname(use_long_names: bool) -> Result<String> {
    let full_hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string());

    if use_long_names {
        Ok(full_hostname)
    } else {
        Ok(full_hostname
            .split('.')
            .next()
            .unwrap_or(&full_hostname)
            .to_string())
    }
}

/// Builds a client node name with the given prefix and remote host
pub fn build_client_node_name(prefix: &str, remote_host: &str) -> String {
    let use_long_names = remote_host.contains('.');
    let hostname = get_hostname(use_long_names).unwrap_or_else(|_| "localhost".to_string());
    format!("{}@{}", prefix, hostname)
}
