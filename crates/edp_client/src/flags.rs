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

//! Distribution protocol capability flags for Erlang/OTP 26+.

use bitflags::bitflags;

bitflags! {
    /// Distribution capability flags as u64 bitmask.
    ///
    /// These flags are negotiated during the handshake to determine which
    /// protocol features are supported by both nodes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DistributionFlags: u64 {
        /// Published node (not hidden)
        const PUBLISHED = 0x01;

        /// Use atom cache in distribution messages
        const ATOM_CACHE = 0x02;

        /// Extended 3×32-bit references (mandatory in Erlang/OTP 26+)
        const EXTENDED_REFERENCES = 0x04;

        /// Distributed process monitoring support
        const DIST_MONITOR = 0x08;

        /// Separate function tags (mandatory in Erlang/OTP 26+)
        const FUN_TAGS = 0x10;

        /// Distributed named process monitoring
        const DIST_MONITOR_NAME = 0x20;

        /// Hidden node (not published in global name space)
        const HIDDEN_ATOM_CACHE = 0x40;

        /// NEW_FUN_EXT understanding (mandatory in Erlang/OTP 26+)
        const NEW_FUN_TAGS = 0x80;

        /// Extended PID and port handling (mandatory in Erlang/OTP 26+)
        const EXTENDED_PIDS_PORTS = 0x100;

        /// EXPORT_EXT tag support (mandatory in Erlang/OTP 26+)
        const EXPORT_PTR_TAG = 0x200;

        /// BIT_BINARY_EXT tag support (mandatory in Erlang/OTP 26+)
        const BIT_BINARIES = 0x400;

        /// NEW_FLOAT_EXT understanding (mandatory in Erlang/OTP 26+)
        const NEW_FLOATS = 0x800;

        /// Support for EPMD ALIVE2_X_RESP (32-bit creation)
        const DIST_HDR_ATOM_CACHE = 0x2000;

        /// Small atom tags support
        const SMALL_ATOM_TAGS = 0x4000;

        /// UTF-8 atom encoding (mandatory in Erlang/OTP 26+)
        const UTF8_ATOMS = 0x10000;

        /// MAP_EXT understanding (mandatory in Erlang/OTP 26+)
        const MAP_TAG = 0x20000;

        /// Large node creation identifiers (mandatory in Erlang/OTP 26+)
        const BIG_CREATION = 0x40000;

        /// Message fragmentation support
        const FRAGMENTS = 0x800_0000;

        /// Handshake version 6 support (mandatory in Erlang/OTP 26+)
        const HANDSHAKE_23 = 0x01000000;

        /// New unlink protocol with IDs (mandatory in Erlang/OTP 26+)
        const UNLINK_ID = 0x02000000;

        /// Spawn operations support
        const SPAWN = 0x10_0000_0000;

        /// Node name in handshake is UTF-8
        const NAME_ME = 0x20_0000_0000;

        /// Version 4 node containers (mandatory in Erlang/OTP 26+)
        const V4_NC = 0x04_0000_0000;

        /// Process alias support
        const ALIAS = 0x800_0000_0000;
    }
}

impl DistributionFlags {
    /// Mandatory flags for Erlang/OTP 26+ connections.
    ///
    /// All these flags must be set when connecting to OTP 26 or later nodes.
    pub const MANDATORY_OTP26: Self = Self::from_bits_truncate(
        Self::EXTENDED_REFERENCES.bits()
            | Self::FUN_TAGS.bits()
            | Self::NEW_FUN_TAGS.bits()
            | Self::EXTENDED_PIDS_PORTS.bits()
            | Self::EXPORT_PTR_TAG.bits()
            | Self::BIT_BINARIES.bits()
            | Self::NEW_FLOATS.bits()
            | Self::UTF8_ATOMS.bits()
            | Self::MAP_TAG.bits()
            | Self::BIG_CREATION.bits()
            | Self::HANDSHAKE_23.bits()
            | Self::UNLINK_ID.bits()
            | Self::V4_NC.bits(),
    );

    /// Default flags for a modern Rust client targeting OTP 26+.
    ///
    /// Includes all mandatory flags plus commonly used optional features.
    /// This creates a published (visible) node.
    pub const DEFAULT: Self = Self::from_bits_truncate(
        Self::MANDATORY_OTP26.bits()
            | Self::PUBLISHED.bits()
            | Self::DIST_MONITOR.bits()
            | Self::DIST_MONITOR_NAME.bits()
            | Self::SMALL_ATOM_TAGS.bits()
            | Self::FRAGMENTS.bits()
            | Self::SPAWN.bits()
            | Self::NAME_ME.bits()
            | Self::ALIAS.bits(),
    );

    /// Default flags for a hidden node targeting OTP 26+.
    ///
    /// Same as DEFAULT but without PUBLISHED flag, making this a hidden node.
    /// Hidden nodes don't participate in global name registration and are
    /// typically used for client connections that don't want cluster messages.
    pub const DEFAULT_HIDDEN: Self = Self::from_bits_truncate(
        Self::MANDATORY_OTP26.bits()
            | Self::DIST_MONITOR.bits()
            | Self::DIST_MONITOR_NAME.bits()
            | Self::SMALL_ATOM_TAGS.bits()
            | Self::FRAGMENTS.bits()
            | Self::SPAWN.bits()
            | Self::NAME_ME.bits()
            | Self::ALIAS.bits(),
    );

    /// Create a new flags instance with the given bitmask.
    pub const fn new(flags: u64) -> Self {
        Self::from_bits_retain(flags)
    }

    /// Create flags with default settings for OTP 26+.
    pub const fn default_otp26() -> Self {
        Self::DEFAULT
    }

    /// Create flags for a hidden node targeting OTP 26+.
    pub const fn default_hidden() -> Self {
        Self::DEFAULT_HIDDEN
    }

    /// Check if a specific flag is set.
    pub const fn has(&self, flag: Self) -> bool {
        self.contains(flag)
    }

    /// Check if all mandatory OTP 26 flags are set.
    pub const fn has_mandatory_otp26(&self) -> bool {
        self.contains(Self::MANDATORY_OTP26)
    }

    /// Get the raw u64 value.
    pub const fn as_u64(&self) -> u64 {
        self.bits()
    }
}

impl Default for DistributionFlags {
    fn default() -> Self {
        Self::default_otp26()
    }
}

impl From<u64> for DistributionFlags {
    fn from(flags: u64) -> Self {
        Self::from_bits_retain(flags)
    }
}

impl From<DistributionFlags> for u64 {
    fn from(flags: DistributionFlags) -> u64 {
        flags.bits()
    }
}
