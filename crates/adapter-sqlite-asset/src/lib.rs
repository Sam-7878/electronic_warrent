//! Transactional SQLite implementation of the policy-aware asset adapter.
//!
//! `prepare` is read-only. `commit` performs the version check, state update,
//! command-id insertion, and receipt persistence in one SQLite transaction.

use std::collections::BTreeMap;
use std::path::Path;

use hete_adapter_api::{
    AdapterError, AdapterId, AdapterManifest, EnforcementAdapter, EnforcementCommand,
    ExecutionReceipt, PreparedChange, ResourceSnapshot, require_capabilities,
};
use hete_identity::Timestamp;
use hete_policy::{EnforcementAction, ResourceKind, ResourceRef};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LedgerEntry {
    resource: ResourceRef,
    balance: u128,
    reservations: BTreeMap<String, Reservation>,
    version: u64,
}

pub struct SqliteAssetAdapter {
    connection: Connection,
    now: Timestamp,
}

impl SqliteAssetAdapter {
    pub fn open(path: impl AsRef<Path>, now: Timestamp) -> Result<Self, AdapterError> {
        let connection = Connection::open(path).map_err(|_| AdapterError::PrepareFailed)?;
        connection
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA synchronous=FULL;
                 PRAGMA foreign_keys=ON;
                 CREATE TABLE IF NOT EXISTS resources (
                   resource_id TEXT PRIMARY KEY,
                   entry_json TEXT NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS committed_commands (
                   command_id TEXT PRIMARY KEY,
                   receipt_json TEXT NOT NULL
                 );",
            )
            .map_err(|_| AdapterError::PrepareFailed)?;
        Ok(Self { connection, now })
    }

    pub fn in_memory(now: Timestamp) -> Result<Self, AdapterError> {
        Self::open(":memory:", now)
    }

    pub fn register_resource(
        &mut self,
        resource: ResourceRef,
        balance: u128,
    ) -> Result<(), AdapterError> {
        let entry = LedgerEntry {
            resource: resource.clone(),
            balance,
            reservations: BTreeMap::new(),
            version: 0,
        };
        let json = serde_json::to_string(&entry).map_err(|_| AdapterError::CommitFailed)?;
        self.connection
            .execute(
                "INSERT INTO resources(resource_id, entry_json) VALUES (?1, ?2)
                 ON CONFLICT(resource_id) DO UPDATE SET entry_json=excluded.entry_json",
                params![resource.resource_id, json],
            )
            .map_err(|_| AdapterError::CommitFailed)?;
        Ok(())
    }

    pub fn set_time(&mut self, now: Timestamp) {
        self.now = now;
    }

    pub fn committed_command_count(&self) -> Result<u64, AdapterError> {
        self.connection
            .query_row("SELECT COUNT(*) FROM committed_commands", [], |row| {
                row.get(0)
            })
            .map_err(|_| AdapterError::PrepareFailed)
    }

    fn manifest_value() -> AdapterManifest {
        AdapterManifest {
            adapter_id: AdapterId("sqlite-transactional-asset".into()),
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
            assurance: "sqlite-acid-wal-full-sync-reference".into(),
        }
    }

    fn read_entry(connection: &Connection, resource_id: &str) -> Result<LedgerEntry, AdapterError> {
        let json: Option<String> = connection
            .query_row(
                "SELECT entry_json FROM resources WHERE resource_id=?1",
                [resource_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| AdapterError::PrepareFailed)?;
        serde_json::from_str(&json.ok_or(AdapterError::ResourceNotFound)?)
            .map_err(|_| AdapterError::PrepareFailed)
    }

    fn snapshot(entry: &LedgerEntry, now: Timestamp) -> Result<ResourceSnapshot, AdapterError> {
        Ok(ResourceSnapshot {
            resource: entry.resource.clone(),
            balance: Some(entry.balance),
            active_reserved_amount: entry.reservations.values().map(|r| r.active(now)).sum(),
            pending_execution_amount: 0,
            version: entry.version,
            state_digest: digest(entry)?,
        })
    }

    fn apply(entry: &mut LedgerEntry, command: &EnforcementCommand) -> Result<(), AdapterError> {
        match command.action {
            EnforcementAction::Freeze => {
                if command.amount == 0
                    || command.amount > entry.balance
                    || command.effective_at >= command.expires_at
                {
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

impl EnforcementAdapter for SqliteAssetAdapter {
    fn manifest(&self) -> AdapterManifest {
        Self::manifest_value()
    }

    fn inspect(&self, resource: &ResourceRef) -> Result<ResourceSnapshot, AdapterError> {
        Self::snapshot(
            &Self::read_entry(&self.connection, &resource.resource_id)?,
            self.now,
        )
    }

    fn prepare(&mut self, command: &EnforcementCommand) -> Result<PreparedChange, AdapterError> {
        require_capabilities(
            &Self::manifest_value(),
            &command.resource.kind,
            &command.action,
        )?;
        let duplicate: bool = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM committed_commands WHERE command_id=?1)",
                [&command.command_id],
                |row| row.get(0),
            )
            .map_err(|_| AdapterError::PrepareFailed)?;
        if duplicate {
            return Err(AdapterError::DuplicateCommand);
        }
        let entry = Self::read_entry(&self.connection, &command.resource.resource_id)?;
        if entry.version != command.expected_version {
            return Err(AdapterError::StaleSnapshot);
        }
        let before = Self::snapshot(&entry, self.now)?;
        let mut candidate_entry = entry;
        Self::apply(&mut candidate_entry, command)?;
        let candidate = Self::snapshot(&candidate_entry, self.now)?;
        Ok(PreparedChange {
            command: command.clone(),
            preparation_digest: digest(&(command, &before.state_digest, &candidate.state_digest))?,
            before,
            candidate,
        })
    }

    fn commit(&mut self, prepared: PreparedChange) -> Result<ExecutionReceipt, AdapterError> {
        let now = self.now;
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| AdapterError::CommitFailed)?;
        let duplicate: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM committed_commands WHERE command_id=?1)",
                [&prepared.command.command_id],
                |row| row.get(0),
            )
            .map_err(|_| AdapterError::CommitFailed)?;
        if duplicate {
            return Err(AdapterError::DuplicateCommand);
        }
        let entry = Self::read_entry(&transaction, &prepared.command.resource.resource_id)?;
        if entry.version != prepared.before.version
            || digest(&entry)? != prepared.before.state_digest
        {
            return Err(AdapterError::StaleSnapshot);
        }
        let mut committed = entry;
        Self::apply(&mut committed, &prepared.command)?;
        let after = Self::snapshot(&committed, now)?;
        if after != prepared.candidate {
            return Err(AdapterError::CommitFailed);
        }
        let entry_json =
            serde_json::to_string(&committed).map_err(|_| AdapterError::CommitFailed)?;
        transaction
            .execute(
                "UPDATE resources SET entry_json=?1 WHERE resource_id=?2",
                params![entry_json, prepared.command.resource.resource_id],
            )
            .map_err(|_| AdapterError::CommitFailed)?;
        let receipt = ExecutionReceipt {
            receipt_id: digest(&(
                &prepared.command.command_id,
                &prepared.before.state_digest,
                &after.state_digest,
            ))?,
            command_id: prepared.command.command_id.clone(),
            adapter_id: Self::manifest_value().adapter_id,
            adapter_version: Self::manifest_value().version,
            before_digest: prepared.before.state_digest,
            after_digest: after.state_digest,
            committed_at: now,
            dry_run: false,
        };
        let receipt_json =
            serde_json::to_string(&receipt).map_err(|_| AdapterError::CommitFailed)?;
        transaction
            .execute(
                "INSERT INTO committed_commands(command_id, receipt_json) VALUES (?1, ?2)",
                params![receipt.command_id, receipt_json],
            )
            .map_err(|_| AdapterError::DuplicateCommand)?;
        transaction
            .commit()
            .map_err(|_| AdapterError::CommitFailed)?;
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

fn digest<T: Serialize>(value: &T) -> Result<String, AdapterError> {
    let bytes = poa_protocol::canonicalize_value(value).map_err(|_| AdapterError::CommitFailed)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hete_policy::PseudonymousTargetRef;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn resource() -> ResourceRef {
        ResourceRef {
            resource_id: "account-1".into(),
            kind: ResourceKind::Account,
            target: PseudonymousTargetRef::derive("test", "subject", "account-1", "w1", b"salt"),
        }
    }

    fn command(
        id: &str,
        action: EnforcementAction,
        amount: u128,
        version: u64,
    ) -> EnforcementCommand {
        EnforcementCommand {
            command_id: id.into(),
            warrant_id: "w1".into(),
            resource: resource(),
            action,
            amount,
            effective_at: 10,
            expires_at: 20,
            expected_version: version,
        }
    }

    fn adapter() -> SqliteAssetAdapter {
        let mut adapter = SqliteAssetAdapter::in_memory(10).unwrap();
        adapter.register_resource(resource(), 100).unwrap();
        adapter
    }

    #[test]
    fn freeze_is_atomic_and_bounded() {
        let mut adapter = adapter();
        let prepared = adapter
            .prepare(&command("c1", EnforcementAction::Freeze, 60, 0))
            .unwrap();
        adapter.commit(prepared).unwrap();
        assert_eq!(
            adapter
                .inspect(&resource())
                .unwrap()
                .available_to_transfer(),
            Some(40)
        );
    }

    #[test]
    fn prepare_is_side_effect_free_and_rollback_is_noop() {
        let mut adapter = adapter();
        let prepared = adapter
            .prepare(&command("c1", EnforcementAction::Freeze, 60, 0))
            .unwrap();
        assert_eq!(adapter.inspect(&resource()).unwrap().version, 0);
        adapter.rollback(prepared).unwrap();
        assert_eq!(adapter.inspect(&resource()).unwrap().version, 0);
    }

    #[test]
    fn stale_commit_has_no_partial_publication() {
        let mut adapter = adapter();
        let stale = adapter
            .prepare(&command("stale", EnforcementAction::Freeze, 40, 0))
            .unwrap();
        let fresh = adapter
            .prepare(&command("fresh", EnforcementAction::Freeze, 30, 0))
            .unwrap();
        adapter.commit(fresh).unwrap();
        assert_eq!(adapter.commit(stale), Err(AdapterError::StaleSnapshot));
        assert_eq!(adapter.committed_command_count().unwrap(), 1);
    }

    #[test]
    fn duplicate_command_is_idempotently_rejected() {
        let mut adapter = adapter();
        let prepared = adapter
            .prepare(&command("c1", EnforcementAction::Freeze, 60, 0))
            .unwrap();
        adapter.commit(prepared.clone()).unwrap();
        assert_eq!(
            adapter.commit(prepared),
            Err(AdapterError::DuplicateCommand)
        );
    }

    #[test]
    fn execute_and_release_conserve_reservation() {
        let mut adapter = adapter();
        let freeze = adapter
            .prepare(&command("c1", EnforcementAction::Freeze, 60, 0))
            .unwrap();
        adapter.commit(freeze).unwrap();
        let execute = adapter
            .prepare(&command("c2", EnforcementAction::Execute, 20, 1))
            .unwrap();
        adapter.commit(execute).unwrap();
        let release = adapter
            .prepare(&command("c3", EnforcementAction::Release, 0, 2))
            .unwrap();
        adapter.commit(release).unwrap();
        let snapshot = adapter.inspect(&resource()).unwrap();
        assert_eq!(snapshot.balance, Some(80));
        assert_eq!(snapshot.active_reserved_amount, 0);
    }

    #[test]
    fn expiry_releases_availability() {
        let mut adapter = adapter();
        let freeze = adapter
            .prepare(&command("c1", EnforcementAction::Freeze, 60, 0))
            .unwrap();
        adapter.commit(freeze).unwrap();
        adapter.set_time(20);
        assert_eq!(
            adapter
                .inspect(&resource())
                .unwrap()
                .available_to_transfer(),
            Some(100)
        );
    }

    #[test]
    fn file_database_recovers_after_restart() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("hete-sqlite-{unique}.db"));
        {
            let mut adapter = SqliteAssetAdapter::open(&path, 10).unwrap();
            adapter.register_resource(resource(), 100).unwrap();
            let prepared = adapter
                .prepare(&command("c1", EnforcementAction::Freeze, 60, 0))
                .unwrap();
            adapter.commit(prepared).unwrap();
        }
        let adapter = SqliteAssetAdapter::open(&path, 10).unwrap();
        assert_eq!(adapter.inspect(&resource()).unwrap().version, 1);
        assert_eq!(adapter.committed_command_count().unwrap(), 1);
        drop(adapter);
        let _ = std::fs::remove_file(path);
    }
}
