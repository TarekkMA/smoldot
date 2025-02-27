// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! Type definitions that implement the [`serde::Serialize`] and [`serde::Deserialize`] traits and
//! that match the chain specs JSON file structure.
//!
//! The main type is [`ClientSpec`].

use super::light_sync_state::LightSyncState;

use alloc::{boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};
use fnv::FnvBuildHasher;
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(super) struct ClientSpec {
    pub(super) name: String,
    pub(super) id: String,
    #[serde(default)]
    pub(super) chain_type: ChainType,

    /// Mapping from a block number to a hex-encoded wasm runtime code (normally found in the
    /// `:code` storage key).
    ///
    /// The given runtime code will be used to substitute the on-chain runtime code starting with
    /// the given block number until the `spec_version`
    /// ([`crate::executor::CoreVersionRef::spec_version`]) on chain changes.
    #[serde(default)]
    // TODO: make use of this
    pub(super) code_substitutes: HashMap<u64, HexString, fnv::FnvBuildHasher>,
    pub(super) boot_nodes: Vec<String>,
    pub(super) telemetry_endpoints: Option<Vec<(String, u8)>>,
    pub(super) protocol_id: Option<String>,
    #[serde(default = "Default::default", skip_serializing_if = "Option::is_none")]
    pub(super) fork_id: Option<String>,
    /// The `blockNumberBytes` field is (at the time of writing of this comment) a custom addition
    /// to the format of smoldot chain specs compared to Substrate. It is necessary because,
    /// contrary to Substrate, smoldot has no way to know the size of the block number field of
    /// various data structures. If the field is missing, a value of 4 is assumed.
    // TODO: revisit this field in the future to maybe bring compatibility with Substrate
    #[serde(default = "Default::default", skip_serializing_if = "Option::is_none")]
    pub(super) block_number_bytes: Option<u8>,
    pub(super) properties: Option<Box<serde_json::value::RawValue>>,
    // TODO: make use of this
    pub(super) fork_blocks: Option<Vec<(u64, HashHexString)>>,
    // TODO: make use of this
    pub(super) bad_blocks: Option<HashSet<HashHexString, FnvBuildHasher>>,
    // Unused but for some reason still part of the chain specs.
    #[serde(default, skip_serializing)]
    #[allow(unused)]
    pub(super) consensus_engine: (),
    pub(super) genesis: Genesis,
    pub(super) light_sync_state: Option<LightSyncState>,
    #[serde(flatten)]
    pub(super) parachain: Option<ChainSpecParachain>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct ChainSpecParachain {
    // Note that in Substrate/Cumulus this field is only named `relay_chain` and `relayChain` is
    // not accepted (as of 2022-06-09). This seems to be an oversight, as there are only two
    // fields that use snake_case while the rest uses camelCase. For this reason, smoldot
    // supports both.
    #[serde(alias = "relayChain")]
    pub(super) relay_chain: String,
    // Same remark concerning the name as `relay_chain`
    #[serde(alias = "paraId")]
    pub(super) para_id: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) enum ChainType {
    Development,
    Local,
    Live,
    Custom(String),
}

impl Default for ChainType {
    fn default() -> Self {
        Self::Live
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(super) enum Genesis {
    Raw(RawGenesis),
    StateRootHash(HashHexString),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(super) struct RawGenesis {
    pub(super) top: BTreeMap<HexString, HexString>,
    pub(super) children_default: BTreeMap<HexString, ChildRawStorage>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct HexString(pub(super) Vec<u8>);

impl core::borrow::Borrow<[u8]> for HexString {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl serde::Serialize for HexString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("0x{}", hex::encode(&self.0[..])).serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for HexString {
    fn deserialize<D>(deserializer: D) -> Result<HexString, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let string = String::deserialize(deserializer)?;

        if let Some(hex) = string.strip_prefix("0x") {
            let bytes = hex::decode(&hex).map_err(serde::de::Error::custom)?;
            return Ok(HexString(bytes));
        }

        Err(serde::de::Error::custom(
            "hexadecimal string doesn't start with 0x",
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(super) struct ChildRawStorage {
    pub(super) child_info: Vec<u8>,
    pub(super) child_type: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct HashHexString(pub(super) [u8; 32]);

impl serde::Serialize for HashHexString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("0x{}", hex::encode(&self.0[..])).serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for HashHexString {
    fn deserialize<D>(deserializer: D) -> Result<HashHexString, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let string = String::deserialize(deserializer)?;

        if !string.starts_with("0x") {
            return Err(serde::de::Error::custom("hash doesn't start with 0x"));
        }

        let bytes = hex::decode(&string[2..]).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::invalid_length(
                bytes.len(),
                &"a 32 bytes hash",
            ));
        }

        let mut out = [0; 32];
        out.copy_from_slice(&bytes);
        Ok(HashHexString(out))
    }
}
