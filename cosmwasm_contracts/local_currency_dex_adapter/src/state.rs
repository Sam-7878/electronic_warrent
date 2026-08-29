use cw_storage_plus::{Item, Map};
use crate::msg::{AgentInfo, CredentialInfo, TransferRecord};

/// Registered DID agents (key: DID string)
pub const AGENTS: Map<&str, AgentInfo> = Map::new("agents");

/// Issued credentials (key: (subject_did, credential_type))
pub const CREDENTIALS: Map<(&str, &str), CredentialInfo> = Map::new("credentials");

/// Transfer history log
pub const TRANSFER_LOG: Item<Vec<TransferRecord>> = Item::new("transfer_log");

/// Default region code (set during instantiation)
pub const DEFAULT_REGION: Item<String> = Item::new("default_region");

use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WarrantInfo {
    pub is_frozen: bool,
    pub frozen_amount: Uint128,
    pub expiration_time: u64,
}

/// Warrant registry by target DID hash (key: SHA256(target_did || warrant_id || salt))
pub const WARRANT_REGISTRY: Map<&[u8], WarrantInfo> = Map::new("warrant_registry");

/// Active commitments (key: warrant_hash, value: block_height)
pub const WARRANT_COMMITMENTS: Map<&[u8], u64> = Map::new("warrant_commitments");

/// Pending commitments by target DID hash (key: SHA256(target_did), value: list of warrant hashes)
pub const PENDING_COMMITMENTS: Map<&[u8], Vec<Binary>> = Map::new("pending_commitments");

/// Active warrants by target DID hash (key: SHA256(target_did), value: list of warrant hashes)
pub const ACTIVE_WARRANTS_BY_DID: Map<&[u8], Vec<Binary>> = Map::new("active_warrants_by_did");

