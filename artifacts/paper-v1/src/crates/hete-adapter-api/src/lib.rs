//! Replaceable enforcement adapter contract.
//!
//! `prepare` must not publish state. `commit` must reject a stale snapshot and
//! apply one complete prepared change or none. Unsupported capabilities always
//! fail closed.

use hete_identity::Timestamp;
use hete_policy::{EnforcementAction, ResourceKind, ResourceRef};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AdapterId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterManifest {
    pub adapter_id: AdapterId,
    pub version: String,
    pub supported_resources: Vec<ResourceKind>,
    pub supported_actions: Vec<EnforcementAction>,
    pub supports_atomic_prepare_commit: bool,
    pub supports_amount_bounded_freeze: bool,
    pub supports_expiration: bool,
    pub supports_revocation: bool,
    pub authoritative_balance: bool,
    pub assurance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSnapshot {
    pub resource: ResourceRef,
    pub balance: Option<u128>,
    pub active_reserved_amount: u128,
    pub pending_execution_amount: u128,
    pub version: u64,
    pub state_digest: String,
}

impl ResourceSnapshot {
    pub fn available_to_transfer(&self) -> Option<u128> {
        self.balance.map(|balance| {
            balance.saturating_sub(
                self.active_reserved_amount
                    .saturating_add(self.pending_execution_amount),
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnforcementCommand {
    pub command_id: String,
    pub warrant_id: String,
    pub resource: ResourceRef,
    pub action: EnforcementAction,
    pub amount: u128,
    pub effective_at: Timestamp,
    pub expires_at: Timestamp,
    pub expected_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedChange {
    pub command: EnforcementCommand,
    pub before: ResourceSnapshot,
    pub candidate: ResourceSnapshot,
    pub preparation_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionReceipt {
    pub receipt_id: String,
    pub command_id: String,
    pub adapter_id: AdapterId,
    pub adapter_version: String,
    pub before_digest: String,
    pub after_digest: String,
    pub committed_at: Timestamp,
    pub dry_run: bool,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AdapterError {
    #[error("ADAPTER_UNSUPPORTED_CAPABILITY")]
    UnsupportedCapability,
    #[error("ADAPTER_RESOURCE_NOT_FOUND")]
    ResourceNotFound,
    #[error("ADAPTER_NON_AUTHORITATIVE_BALANCE")]
    NonAuthoritativeBalance,
    #[error("ADAPTER_STALE_SNAPSHOT")]
    StaleSnapshot,
    #[error("ADAPTER_PREPARE_FAILED")]
    PrepareFailed,
    #[error("ADAPTER_COMMIT_FAILED")]
    CommitFailed,
    #[error("ADAPTER_DUPLICATE_COMMAND")]
    DuplicateCommand,
    #[error("ADAPTER_AMOUNT_INVALID")]
    InvalidAmount,
}

pub trait EnforcementAdapter {
    fn manifest(&self) -> AdapterManifest;
    fn inspect(&self, resource: &ResourceRef) -> Result<ResourceSnapshot, AdapterError>;
    fn prepare(&mut self, command: &EnforcementCommand) -> Result<PreparedChange, AdapterError>;
    fn commit(&mut self, prepared: PreparedChange) -> Result<ExecutionReceipt, AdapterError>;
    fn rollback(&mut self, prepared: PreparedChange) -> Result<(), AdapterError>;
}

pub fn require_capabilities(
    manifest: &AdapterManifest,
    resource: &ResourceKind,
    action: &EnforcementAction,
) -> Result<(), AdapterError> {
    if !manifest.supported_resources.contains(resource)
        || !manifest.supported_actions.contains(action)
        || !manifest.supports_atomic_prepare_commit
        || !manifest.authoritative_balance
    {
        return Err(AdapterError::UnsupportedCapability);
    }
    if matches!(action, EnforcementAction::Freeze) && !manifest.supports_amount_bounded_freeze {
        return Err(AdapterError::UnsupportedCapability);
    }
    Ok(())
}
