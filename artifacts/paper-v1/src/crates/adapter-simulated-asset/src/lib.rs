//! Deterministic authoritative ledger used for evaluation.
//!
//! The adapter is process-local and is not a legal or production banking
//! system. It provides optimistic-version atomic prepare/commit behavior and
//! explicit failure injection for invariant testing.

use std::collections::{BTreeMap, BTreeSet};

use hete_adapter_api::{
    AdapterError, AdapterId, AdapterManifest, EnforcementAdapter, EnforcementCommand,
    ExecutionReceipt, PreparedChange, ResourceSnapshot, require_capabilities,
};
use hete_identity::Timestamp;
use hete_policy::{EnforcementAction, ResourceKind, ResourceRef};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Reservation {
    reserved: u128,
    executed: u128,
    released: u128,
    expires_at: Timestamp,
    revoked: bool,
}

impl Reservation {
    fn active(&self, now: Timestamp) -> u128 {
        if self.revoked || now >= self.expires_at {
            0
        } else {
            self.reserved
                .saturating_sub(self.executed.saturating_add(self.released))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LedgerEntry {
    resource: ResourceRef,
    balance: u128,
    reservations: BTreeMap<String, Reservation>,
    version: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FailureInjection {
    pub fail_prepare: bool,
    pub fail_commit: bool,
}

#[derive(Debug, Clone)]
pub struct SimulatedAssetAdapter {
    entries: BTreeMap<String, LedgerEntry>,
    committed_commands: BTreeSet<String>,
    history: Vec<ExecutionReceipt>,
    now: Timestamp,
    failure: FailureInjection,
}

impl SimulatedAssetAdapter {
    pub fn new(now: Timestamp) -> Self {
        Self {
            entries: BTreeMap::new(),
            committed_commands: BTreeSet::new(),
            history: Vec::new(),
            now,
            failure: FailureInjection::default(),
        }
    }

    pub fn register_resource(&mut self, resource: ResourceRef, balance: u128) {
        self.entries.insert(
            resource.resource_id.clone(),
            LedgerEntry {
                resource,
                balance,
                reservations: BTreeMap::new(),
                version: 0,
            },
        );
    }

    pub fn set_time(&mut self, now: Timestamp) {
        self.now = now;
    }

    pub fn set_failure_injection(&mut self, failure: FailureInjection) {
        self.failure = failure;
    }

    pub fn history(&self) -> &[ExecutionReceipt] {
        &self.history
    }

    fn manifest_value() -> AdapterManifest {
        AdapterManifest {
            adapter_id: AdapterId("simulated-regulated-asset".into()),
            version: "1.0.0".into(),
            supported_resources: vec![
                ResourceKind::Account,
                ResourceKind::Wallet,
                ResourceKind::Vault,
                ResourceKind::Token,
            ],
            supported_actions: vec![
                EnforcementAction::Freeze,
                EnforcementAction::Release,
                EnforcementAction::Execute,
                EnforcementAction::Inspect,
            ],
            supports_atomic_prepare_commit: true,
            supports_amount_bounded_freeze: true,
            supports_expiration: true,
            supports_revocation: true,
            authoritative_balance: true,
            assurance: "deterministic-process-local-reference".into(),
        }
    }

    fn snapshot_at(entry: &LedgerEntry, now: Timestamp) -> Result<ResourceSnapshot, AdapterError> {
        let active_reserved_amount = entry
            .reservations
            .values()
            .map(|reservation| reservation.active(now))
            .sum();
        Ok(ResourceSnapshot {
            resource: entry.resource.clone(),
            balance: Some(entry.balance),
            active_reserved_amount,
            pending_execution_amount: 0,
            version: entry.version,
            state_digest: digest(entry)?,
        })
    }

    fn apply(entry: &mut LedgerEntry, command: &EnforcementCommand) -> Result<(), AdapterError> {
        match command.action {
            EnforcementAction::Freeze => {
                if command.amount == 0 || command.effective_at >= command.expires_at {
                    return Err(AdapterError::InvalidAmount);
                }
                if entry.reservations.contains_key(&command.warrant_id) {
                    return Err(AdapterError::DuplicateCommand);
                }
                entry.reservations.insert(
                    command.warrant_id.clone(),
                    Reservation {
                        reserved: command.amount,
                        executed: 0,
                        released: 0,
                        expires_at: command.expires_at,
                        revoked: false,
                    },
                );
            }
            EnforcementAction::Execute => {
                let reservation = entry
                    .reservations
                    .get_mut(&command.warrant_id)
                    .ok_or(AdapterError::InvalidAmount)?;
                if command.effective_at >= reservation.expires_at
                    || command.amount == 0
                    || command.amount > reservation.active(command.effective_at)
                    || command.amount > entry.balance
                {
                    return Err(AdapterError::InvalidAmount);
                }
                reservation.executed = reservation.executed.saturating_add(command.amount);
                entry.balance -= command.amount;
            }
            EnforcementAction::Release => {
                let reservation = entry
                    .reservations
                    .get_mut(&command.warrant_id)
                    .ok_or(AdapterError::InvalidAmount)?;
                let remaining = reservation.active(command.effective_at);
                let amount = if command.amount == 0 {
                    remaining
                } else {
                    command.amount
                };
                if amount > remaining {
                    return Err(AdapterError::InvalidAmount);
                }
                reservation.released = reservation.released.saturating_add(amount);
            }
            EnforcementAction::Inspect => {}
            _ => return Err(AdapterError::UnsupportedCapability),
        }
        entry.version = entry.version.saturating_add(1);
        Ok(())
    }
}

impl EnforcementAdapter for SimulatedAssetAdapter {
    fn manifest(&self) -> AdapterManifest {
        Self::manifest_value()
    }

    fn inspect(&self, resource: &ResourceRef) -> Result<ResourceSnapshot, AdapterError> {
        self.entries
            .get(&resource.resource_id)
            .map(|entry| Self::snapshot_at(entry, self.now))
            .ok_or(AdapterError::ResourceNotFound)?
    }

    fn prepare(&mut self, command: &EnforcementCommand) -> Result<PreparedChange, AdapterError> {
        if self.failure.fail_prepare {
            return Err(AdapterError::PrepareFailed);
        }
        require_capabilities(
            &Self::manifest_value(),
            &command.resource.kind,
            &command.action,
        )?;
        if self.committed_commands.contains(&command.command_id) {
            return Err(AdapterError::DuplicateCommand);
        }
        let entry = self
            .entries
            .get(&command.resource.resource_id)
            .ok_or(AdapterError::ResourceNotFound)?;
        if entry.version != command.expected_version {
            return Err(AdapterError::StaleSnapshot);
        }
        let before = Self::snapshot_at(entry, self.now)?;
        let mut candidate_entry = entry.clone();
        Self::apply(&mut candidate_entry, command)?;
        let candidate = Self::snapshot_at(&candidate_entry, self.now)?;
        let preparation_digest = digest(&(command, &before.state_digest, &candidate.state_digest))?;
        Ok(PreparedChange {
            command: command.clone(),
            before,
            candidate,
            preparation_digest,
        })
    }

    fn commit(&mut self, prepared: PreparedChange) -> Result<ExecutionReceipt, AdapterError> {
        if self.failure.fail_commit {
            return Err(AdapterError::CommitFailed);
        }
        if self
            .committed_commands
            .contains(&prepared.command.command_id)
        {
            return Err(AdapterError::DuplicateCommand);
        }
        let entry = self
            .entries
            .get(&prepared.command.resource.resource_id)
            .ok_or(AdapterError::ResourceNotFound)?;
        if entry.version != prepared.before.version
            || digest(entry)? != prepared.before.state_digest
        {
            return Err(AdapterError::StaleSnapshot);
        }
        let mut committed = entry.clone();
        Self::apply(&mut committed, &prepared.command)?;
        let after = Self::snapshot_at(&committed, self.now)?;
        if after != prepared.candidate {
            return Err(AdapterError::CommitFailed);
        }
        self.entries
            .insert(prepared.command.resource.resource_id.clone(), committed);
        self.committed_commands
            .insert(prepared.command.command_id.clone());
        let receipt = ExecutionReceipt {
            receipt_id: digest(&(
                &prepared.command.command_id,
                &prepared.before.state_digest,
                &after.state_digest,
            ))?,
            command_id: prepared.command.command_id,
            adapter_id: Self::manifest_value().adapter_id,
            adapter_version: Self::manifest_value().version,
            before_digest: prepared.before.state_digest,
            after_digest: after.state_digest,
            committed_at: self.now,
            dry_run: false,
        };
        self.history.push(receipt.clone());
        Ok(receipt)
    }

    fn rollback(&mut self, prepared: PreparedChange) -> Result<(), AdapterError> {
        let current = self.inspect(&prepared.command.resource)?;
        if current.version != prepared.before.version {
            return Err(AdapterError::StaleSnapshot);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DryRunAdapter {
    inner: SimulatedAssetAdapter,
    receipts: Vec<ExecutionReceipt>,
}

impl DryRunAdapter {
    pub fn new(inner: SimulatedAssetAdapter) -> Self {
        Self {
            inner,
            receipts: Vec::new(),
        }
    }

    pub fn receipts(&self) -> &[ExecutionReceipt] {
        &self.receipts
    }
}

impl EnforcementAdapter for DryRunAdapter {
    fn manifest(&self) -> AdapterManifest {
        let mut manifest = self.inner.manifest();
        manifest.adapter_id = AdapterId("observer-dry-run".into());
        manifest.assurance = "no-state-change-preview".into();
        manifest
    }

    fn inspect(&self, resource: &ResourceRef) -> Result<ResourceSnapshot, AdapterError> {
        self.inner.inspect(resource)
    }

    fn prepare(&mut self, command: &EnforcementCommand) -> Result<PreparedChange, AdapterError> {
        self.inner.prepare(command)
    }

    fn commit(&mut self, prepared: PreparedChange) -> Result<ExecutionReceipt, AdapterError> {
        let mut preview = self.inner.clone();
        let mut receipt = preview.commit(prepared)?;
        receipt.adapter_id = self.manifest().adapter_id;
        receipt.dry_run = true;
        self.receipts.push(receipt.clone());
        Ok(receipt)
    }

    fn rollback(&mut self, prepared: PreparedChange) -> Result<(), AdapterError> {
        self.inner.rollback(prepared)
    }
}

fn digest<T: Serialize>(value: &T) -> Result<String, AdapterError> {
    let bytes = poa_protocol::canonicalize_value(value).map_err(|_| AdapterError::CommitFailed)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hete_policy::PseudonymousTargetRef;

    fn resource() -> ResourceRef {
        ResourceRef {
            resource_id: "account-1".into(),
            kind: ResourceKind::Account,
            target: PseudonymousTargetRef::derive("test", "subject", "account-1", "w1", b"salt"),
        }
    }

    fn freeze(resource: ResourceRef, version: u64) -> EnforcementCommand {
        EnforcementCommand {
            command_id: "c1".into(),
            warrant_id: "w1".into(),
            resource,
            action: EnforcementAction::Freeze,
            amount: 120,
            effective_at: 10,
            expires_at: 20,
            expected_version: version,
        }
    }

    #[test]
    fn bounded_freeze_saturates_available_balance() {
        let mut adapter = SimulatedAssetAdapter::new(10);
        let resource = resource();
        adapter.register_resource(resource.clone(), 100);
        let prepared = adapter.prepare(&freeze(resource.clone(), 0)).unwrap();
        adapter.commit(prepared).unwrap();
        assert_eq!(
            adapter.inspect(&resource).unwrap().available_to_transfer(),
            Some(0)
        );
    }

    #[test]
    fn commit_failure_has_no_partial_state() {
        let mut adapter = SimulatedAssetAdapter::new(10);
        let resource = resource();
        adapter.register_resource(resource.clone(), 100);
        let prepared = adapter.prepare(&freeze(resource.clone(), 0)).unwrap();
        adapter.set_failure_injection(FailureInjection {
            fail_prepare: false,
            fail_commit: true,
        });
        assert_eq!(adapter.commit(prepared), Err(AdapterError::CommitFailed));
        assert_eq!(adapter.inspect(&resource).unwrap().version, 0);
    }

    #[test]
    fn stale_prepare_is_rejected() {
        let mut adapter = SimulatedAssetAdapter::new(10);
        let resource = resource();
        adapter.register_resource(resource.clone(), 100);
        let first = adapter.prepare(&freeze(resource.clone(), 0)).unwrap();
        let stale = first.clone();
        adapter.commit(first).unwrap();
        assert_eq!(adapter.commit(stale), Err(AdapterError::DuplicateCommand));
    }

    #[test]
    fn dry_run_does_not_change_state() {
        let mut base = SimulatedAssetAdapter::new(10);
        let resource = resource();
        base.register_resource(resource.clone(), 100);
        let mut adapter = DryRunAdapter::new(base);
        let prepared = adapter.prepare(&freeze(resource.clone(), 0)).unwrap();
        assert!(adapter.commit(prepared).unwrap().dry_run);
        assert_eq!(adapter.inspect(&resource).unwrap().version, 0);
    }
}
