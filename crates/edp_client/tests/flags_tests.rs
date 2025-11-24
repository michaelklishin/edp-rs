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

use edp_client::flags::DistributionFlags;

#[test]
fn test_mandatory_flags() {
    let flags = DistributionFlags::default_otp26();
    assert!(flags.has_mandatory_otp26());
    assert!(flags.has(DistributionFlags::EXTENDED_REFERENCES));
    assert!(flags.has(DistributionFlags::UTF8_ATOMS));
    assert!(flags.has(DistributionFlags::HANDSHAKE_23));
    assert!(flags.has(DistributionFlags::V4_NC));
    assert!(flags.has(DistributionFlags::UNLINK_ID));
}

#[test]
fn test_flag_operations() {
    let mut flags = DistributionFlags::new(0);
    assert!(!flags.has(DistributionFlags::PUBLISHED));

    flags.insert(DistributionFlags::PUBLISHED);
    assert!(flags.has(DistributionFlags::PUBLISHED));

    flags.remove(DistributionFlags::PUBLISHED);
    assert!(!flags.has(DistributionFlags::PUBLISHED));
}

#[test]
fn test_default_has_optional_features() {
    let flags = DistributionFlags::default();
    assert!(flags.has(DistributionFlags::FRAGMENTS));
    assert!(flags.has(DistributionFlags::SPAWN));
    assert!(flags.has(DistributionFlags::ALIAS));
    assert!(flags.has(DistributionFlags::DIST_MONITOR));
    assert!(flags.has(DistributionFlags::DIST_MONITOR_NAME));
    assert!(flags.has(DistributionFlags::PUBLISHED));
}

#[test]
fn test_default_hidden_not_published() {
    let flags = DistributionFlags::default_hidden();
    assert!(flags.has(DistributionFlags::FRAGMENTS));
    assert!(!flags.has(DistributionFlags::PUBLISHED));
}
