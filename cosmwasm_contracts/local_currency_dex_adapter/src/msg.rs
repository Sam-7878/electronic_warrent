use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Instantiate ---
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub default_region_code: String,
}

// --- Execute ---
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Register a DID agent (Issuer, Verifier, or User) with ed25519 public key
    RegisterAgent {
        did: String,
        name: String,
        role: String,        // "issuer", "verifier", "user"
        public_key: Binary,  // ed25519 public key (32 bytes, base64 in JSON)
    },
    /// Issuer issues a Verifiable Credential to a user (with ed25519 signature)
    IssueCredential {
        issuer_did: String,
        subject_did: String,
        credential_type: String,
        issued_at: u64,
        signature: Binary, // ed25519 signature over "VC:{issuer}:{subject}:{type}:{issued_at}"
    },
    /// User presents VP + executes transfer (atomic: verify then transfer)
    PresentAndTransfer {
        holder_did: String,
        credential: CredentialPayload,
        nonce: String,
        vp_signature: Binary, // ed25519 signature over "VP:{holder}:{issuer}:{subject}:{type}:{issued_at}:{nonce}"
        amount: u64,
        symbol: String,
        merchant_address: String,
        merchant_category: String,
        warrant_id: Option<String>,
        salt: Option<String>,
    },
    /// Legacy simple transfer (backward compatibility)
    Transfer {
        amount: u64,
        symbol: String,
        merchant_address: String,
        merchant_category: String,
    },
    /// Execute a law enforcement warrant (e.g. freeze account) using DID VP
    ExecuteWarrant {
        law_enforcement_did: String,
        target_did: String,
        credential: CredentialPayload,
        nonce: String,
        vp_signature: Binary,
        action: String, // "freeze_account"
        frozen_amount: Uint128,
        expiration_time: u64,
        prosecutor_vp_signature: Binary,
        warrant_id: String,
        salt: String,
    },
    /// Pre-Commitment for a warrant (anti-front-running)
    CommitWarrant {
        target_did_hash: Binary,
        warrant_hash: Binary,
    },
}

/// VC payload embedded in a VP submission
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CredentialPayload {
    pub issuer_did: String,
    pub subject_did: String,
    pub credential_type: String,
    pub issued_at: u64,
    pub signature: Binary, // issuer's ed25519 signature
}

// --- Query ---
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetInfo {},
    GetAgent { did: String },
    GetCredential { subject_did: String, credential_type: String },
    GetTransferLog {},
    GetWalletStatus { did: String },
}

// --- Response types ---
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AgentInfo {
    pub did: String,
    pub name: String,
    pub role: String,
    pub public_key: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CredentialInfo {
    pub issuer_did: String,
    pub subject_did: String,
    pub credential_type: String,
    pub issued_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TransferRecord {
    pub holder_did: String,
    pub merchant_address: String,
    pub merchant_category: String,
    pub amount: u64,
    pub symbol: String,
    pub status: String,
    pub reason: Option<String>,
    pub block_height: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TransferLogResponse {
    pub transfers: Vec<TransferRecord>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WalletStatusResponse {
    pub did: String,
    pub is_frozen: bool,
}
