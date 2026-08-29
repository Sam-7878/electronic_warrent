//! Electronic asset-freezing warrant reference domain.
//!
//! The crate models machine-verifiable authorization and state enforcement for
//! HETE-aware assets only. It does not determine substantive legal validity and
//! does not claim control over permissionless or non-integrated assets.

use std::collections::{BTreeMap, BTreeSet};

use hete_adapter_api::{
    AdapterError, AdapterId, EnforcementAdapter, EnforcementCommand, ExecutionReceipt,
    PreparedChange, require_capabilities,
};
use hete_credential::{AuthorityApproval, CredentialError, NonceRegistry, verify_authorization};
use hete_identity::{AuthorityRole, LocalIdentityStore, Timestamp};
use hete_policy::{AuthorityRef, EnforcementAction, MachinePolicyObject, Nonce, PolicyDigest};
use poa_core::{
    AbortReason, QuarantinePolicy, RejectReason, RiskAwareAacoHooks, RiskEvidence,
    TransitionDescriptor, TransitionFailure, TransitionOutcome, execute_transition_with_risk,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PseudonymousCaseRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WarrantRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JurisdictionRef(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InflowRule {
    ExistingBalanceOnly,
    IncludeNewInflows,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetScope {
    pub asset_id: String,
    pub maximum_amount: u128,
    pub inflow_rule: InflowRule,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreezeRule {
    pub amount: u128,
    pub effective_from: Timestamp,
    pub expires_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRule {
    pub partial_execution_allowed: bool,
    pub destination_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRule {
    pub review_at: Timestamp,
    pub reviewer_role: AuthorityRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldRule {
    pub maximum_hold_seconds: u64,
    pub holder_role: AuthorityRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetFreezingWarrant {
    pub common: MachinePolicyObject,
    pub case_reference: PseudonymousCaseRef,
    pub warrant_reference: WarrantRef,
    pub jurisdiction: JurisdictionRef,
    pub asset_scope: AssetScope,
    pub freeze_rule: FreezeRule,
    pub execution_rule: ExecutionRule,
    pub requesting_authority: AuthorityRef,
    pub supervisory_authority: Option<AuthorityRef>,
    pub judicial_issuer: AuthorityRef,
    pub review_rule: ReviewRule,
    pub appeal_or_hold_rule: Option<HoldRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreezeStatus {
    Reserved,
    PartiallyExecuted,
    FullyExecuted,
    Released,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreezePosition {
    pub warrant_id: String,
    pub asset_id: String,
    pub reserved_amount: u128,
    pub executed_amount: u128,
    pub released_amount: u128,
    pub effective_from: Timestamp,
    pub expires_at: Timestamp,
    pub status: FreezeStatus,
}

impl FreezePosition {
    pub fn validate(&self, maximum: u128) -> Result<(), WarrantError> {
        if self.reserved_amount > maximum
            || self.executed_amount.saturating_add(self.released_amount) > self.reserved_amount
        {
            return Err(WarrantError::AmountExceeded);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarrantState {
    Draft,
    Submitted,
    CredentialVerified,
    Authorized,
    Scheduled,
    Active,
    PartiallyExecuted,
    FullyExecuted,
    Suspended,
    Revoked,
    Expired,
    Released,
    Rejected,
    Failed,
}

/// Executable-state projection used to compare Rust traces with the TLA+ model.
///
/// Field names intentionally match `formal/tla/ElectronicWarrant.tla`.  This
/// type is evidence plumbing, not a second lifecycle implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormalTraceState {
    pub warrant_state: String,
    pub credentials_verified: bool,
    pub authorized: bool,
    pub domain_valid: bool,
    pub nonce_used: bool,
    pub activation_count: u8,
    pub reserved: u128,
    pub executed: u128,
    pub released: u128,
    pub now: Timestamp,
    pub adapter_committed: bool,
    pub audit_written: bool,
    pub last_action: String,
}

impl WarrantState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::FullyExecuted
                | Self::Revoked
                | Self::Expired
                | Self::Released
                | Self::Rejected
                | Self::Failed
        )
    }

    pub fn allows(self, next: Self) -> bool {
        use WarrantState::*;
        matches!(
            (self, next),
            (Draft, Submitted)
                | (Submitted, CredentialVerified)
                | (CredentialVerified, Authorized)
                | (Authorized, Scheduled | Active | Suspended)
                | (Scheduled, Active | Suspended)
                | (
                    Active,
                    PartiallyExecuted | FullyExecuted | Suspended | Expired | Released
                )
                | (
                    PartiallyExecuted,
                    PartiallyExecuted | FullyExecuted | Suspended | Expired | Released
                )
                | (Suspended, Active | Revoked)
        ) || (!self.is_terminal() && matches!(next, Rejected | Failed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasonCode {
    Commit,
    AuthMissingRequiredRole,
    AuthInvalidSignature,
    AuthReplayNonce,
    AuthDomainMismatch,
    AuthRevokedCredential,
    PolicySchemaInvalid,
    PolicyDigestMismatch,
    PolicyPrivilegeExpansion,
    WarrantInvalidTransition,
    WarrantExpired,
    WarrantRevoked,
    WarrantAmountExceeded,
    WarrantDuplicateExecution,
    AdapterUnsupportedCapability,
    AdapterStaleSnapshot,
    AdapterPrepareFailed,
    AdapterCommitFailed,
    AuditWriteFailed,
    RiskQuarantined,
    SystemAborted,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WarrantError {
    #[error("WARRANT_INVALID_TRANSITION")]
    InvalidTransition,
    #[error("WARRANT_EXPIRED")]
    Expired,
    #[error("WARRANT_REVOKED")]
    Revoked,
    #[error("WARRANT_AMOUNT_EXCEEDED")]
    AmountExceeded,
    #[error("WARRANT_DUPLICATE_EXECUTION")]
    DuplicateExecution,
    #[error("POLICY_INVALID")]
    PolicyInvalid,
    #[error("AUTHORIZATION_FAILED")]
    AuthorizationFailed,
    #[error("ADAPTER_FAILED")]
    AdapterFailed,
}

impl AssetFreezingWarrant {
    pub fn validate(&self, now: Timestamp) -> Result<(), WarrantError> {
        self.common
            .validate(now)
            .map_err(|_| WarrantError::PolicyInvalid)?;
        if self.asset_scope.asset_id != self.common.resource.resource_id
            || self.freeze_rule.amount == 0
            || self.freeze_rule.amount > self.asset_scope.maximum_amount
            || self.freeze_rule.effective_from < self.common.validity.not_before
            || self.freeze_rule.expires_at > self.common.validity.expires_at
            || self.freeze_rule.effective_from >= self.freeze_rule.expires_at
        {
            return Err(WarrantError::AmountExceeded);
        }
        Ok(())
    }

    pub fn signature_message(&self, action: &EnforcementAction) -> Result<Vec<u8>, WarrantError> {
        #[derive(Serialize)]
        struct Message<'a> {
            domain_separator: &'static str,
            environment_id: &'a str,
            policy_digest: &'a PolicyDigest,
            warrant_id: &'a str,
            target_ref: &'a [u8; 32],
            asset_scope: &'a AssetScope,
            maximum_amount: u128,
            not_before: Timestamp,
            expires_at: Timestamp,
            action: &'a EnforcementAction,
            nonce: &'a Nonce,
        }
        poa_protocol::canonicalize_value(&Message {
            domain_separator: "HETE-EW-V1",
            environment_id: &self.common.domain_binding.environment_id,
            policy_digest: &self.common.policy_digest,
            warrant_id: &self.warrant_reference.0,
            target_ref: &self.common.subject.target.digest,
            asset_scope: &self.asset_scope,
            maximum_amount: self.asset_scope.maximum_amount,
            not_before: self.freeze_rule.effective_from,
            expires_at: self.freeze_rule.expires_at,
            action,
            nonce: &self.common.nonce,
        })
        .map_err(|_| WarrantError::PolicyInvalid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnforcementEvidence {
    pub audit_id: String,
    pub policy_digest: PolicyDigest,
    pub transition_id: String,
    pub actor_refs: Vec<String>,
    pub authority_roles: Vec<AuthorityRole>,
    pub adapter_id: AdapterId,
    pub adapter_version: String,
    pub before_digest: String,
    pub candidate_digest: String,
    pub after_digest: Option<String>,
    pub outcome: String,
    pub reason_code: ReasonCode,
    pub timestamp: Timestamp,
    pub run_id: String,
    pub source_commit: String,
    pub previous_hash: Option<String>,
    pub record_hash: String,
}

#[derive(Debug, Default, Clone)]
pub struct AuditChain(Vec<EnforcementEvidence>);

impl AuditChain {
    pub fn records(&self) -> &[EnforcementEvidence] {
        &self.0
    }

    pub fn append(&mut self, mut evidence: EnforcementEvidence) -> Result<(), WarrantError> {
        evidence.previous_hash = self.0.last().map(|record| record.record_hash.clone());
        evidence.record_hash.clear();
        evidence.record_hash = digest(&evidence)?;
        self.0.push(evidence);
        Ok(())
    }

    pub fn verify(&self) -> bool {
        let mut previous: Option<&str> = None;
        for record in &self.0 {
            if record.previous_hash.as_deref() != previous {
                return false;
            }
            let mut unsigned = record.clone();
            unsigned.record_hash.clear();
            if digest(&unsigned).ok().as_deref() != Some(record.record_hash.as_str()) {
                return false;
            }
            previous = Some(&record.record_hash);
        }
        true
    }
}

#[derive(Debug, Clone)]
pub struct WarrantRecord {
    pub warrant: AssetFreezingWarrant,
    pub state: WarrantState,
    pub position: Option<FreezePosition>,
    pub last_receipt: Option<ExecutionReceipt>,
}

impl WarrantRecord {
    /// Project the authoritative Rust record onto the variables checked by TLC.
    pub fn formal_trace_state(
        &self,
        now: Timestamp,
        audit_written: bool,
        last_action: impl Into<String>,
    ) -> FormalTraceState {
        let position = self.position.as_ref();
        let credentials_verified = !matches!(
            self.state,
            WarrantState::Draft | WarrantState::Submitted | WarrantState::Rejected
        );
        let authorized = matches!(
            self.state,
            WarrantState::Authorized
                | WarrantState::Scheduled
                | WarrantState::Active
                | WarrantState::PartiallyExecuted
                | WarrantState::FullyExecuted
                | WarrantState::Suspended
                | WarrantState::Revoked
                | WarrantState::Expired
                | WarrantState::Released
        );
        FormalTraceState {
            warrant_state: format!("{:?}", self.state),
            credentials_verified,
            authorized,
            domain_valid: true,
            nonce_used: authorized,
            activation_count: u8::from(position.is_some()),
            reserved: position.map_or(0, |value| value.reserved_amount),
            executed: position.map_or(0, |value| value.executed_amount),
            released: position.map_or(0, |value| value.released_amount),
            now,
            adapter_committed: position.is_some(),
            audit_written,
            last_action: last_action.into(),
        }
    }

    /// Apply a lifecycle transition after the corresponding authorized adapter
    /// operation has succeeded. Invalid or terminal-state resurrection fails closed.
    pub fn transition(&mut self, next: WarrantState) -> Result<(), WarrantError> {
        if !self.state.allows(next) {
            return Err(WarrantError::InvalidTransition);
        }
        self.state = next;
        Ok(())
    }

    pub fn record_execution(&mut self, amount: u128) -> Result<(), WarrantError> {
        if !matches!(
            self.state,
            WarrantState::Active | WarrantState::PartiallyExecuted
        ) {
            return Err(WarrantError::InvalidTransition);
        }
        let position = self
            .position
            .as_mut()
            .ok_or(WarrantError::InvalidTransition)?;
        let next_executed = position.executed_amount.saturating_add(amount);
        if amount == 0
            || next_executed.saturating_add(position.released_amount) > position.reserved_amount
        {
            return Err(WarrantError::AmountExceeded);
        }
        let next = if next_executed == position.reserved_amount {
            WarrantState::FullyExecuted
        } else {
            WarrantState::PartiallyExecuted
        };
        if !self.state.allows(next) {
            return Err(WarrantError::InvalidTransition);
        }
        position.executed_amount = next_executed;
        position.status = if matches!(next, WarrantState::FullyExecuted) {
            FreezeStatus::FullyExecuted
        } else {
            FreezeStatus::PartiallyExecuted
        };
        self.state = next;
        Ok(())
    }

    pub fn record_release(&mut self, amount: u128) -> Result<(), WarrantError> {
        if !self.state.allows(WarrantState::Released) {
            return Err(WarrantError::InvalidTransition);
        }
        let position = self
            .position
            .as_mut()
            .ok_or(WarrantError::InvalidTransition)?;
        let remaining = position.reserved_amount.saturating_sub(
            position
                .executed_amount
                .saturating_add(position.released_amount),
        );
        let release = if amount == 0 { remaining } else { amount };
        if release > remaining {
            return Err(WarrantError::AmountExceeded);
        }
        position.released_amount = position.released_amount.saturating_add(release);
        position.status = FreezeStatus::Released;
        self.state = WarrantState::Released;
        Ok(())
    }

    pub fn expire(&mut self, now: Timestamp) -> Result<(), WarrantError> {
        let position = self
            .position
            .as_mut()
            .ok_or(WarrantError::InvalidTransition)?;
        if now < position.expires_at || !self.state.allows(WarrantState::Expired) {
            return Err(WarrantError::InvalidTransition);
        }
        position.released_amount = position
            .reserved_amount
            .saturating_sub(position.executed_amount);
        position.status = FreezeStatus::Expired;
        self.state = WarrantState::Expired;
        Ok(())
    }

    pub fn revoke(&mut self) -> Result<(), WarrantError> {
        if !self.state.allows(WarrantState::Suspended) {
            return Err(WarrantError::InvalidTransition);
        }
        self.state = WarrantState::Suspended;
        self.transition(WarrantState::Revoked)?;
        if let Some(position) = self.position.as_mut() {
            position.released_amount = position
                .reserved_amount
                .saturating_sub(position.executed_amount);
            position.status = FreezeStatus::Revoked;
        }
        Ok(())
    }
}

pub struct WarrantService<A> {
    adapter: A,
    records: BTreeMap<String, WarrantRecord>,
    nonces: NonceRegistry,
    audit: AuditChain,
    pub run_id: String,
    pub source_commit: String,
}

#[derive(Debug, Clone)]
pub struct EnforcementResult {
    pub outcome: TransitionOutcome,
    pub reason_code: ReasonCode,
    pub receipt: Option<ExecutionReceipt>,
}

impl<A: EnforcementAdapter> WarrantService<A> {
    pub fn new(adapter: A, run_id: impl Into<String>, source_commit: impl Into<String>) -> Self {
        Self {
            adapter,
            records: BTreeMap::new(),
            nonces: NonceRegistry::default(),
            audit: AuditChain::default(),
            run_id: run_id.into(),
            source_commit: source_commit.into(),
        }
    }

    pub fn adapter(&self) -> &A {
        &self.adapter
    }

    pub fn record(&self, warrant_id: &str) -> Option<&WarrantRecord> {
        self.records.get(warrant_id)
    }

    pub fn audit(&self) -> &AuditChain {
        &self.audit
    }

    #[allow(clippy::too_many_arguments)]
    pub fn activate(
        &mut self,
        warrant: AssetFreezingWarrant,
        approvals: &[AuthorityApproval],
        identities: &LocalIdentityStore,
        now: Timestamp,
        risk: Option<RiskEvidence>,
        risk_policy: Option<&QuarantinePolicy>,
    ) -> EnforcementResult {
        let warrant_id = warrant.warrant_reference.0.clone();
        if self.records.contains_key(&warrant_id) {
            return self.early_outcome(&warrant, now, ReasonCode::WarrantDuplicateExecution);
        }
        if warrant.validate(now).is_err() {
            return self.early_outcome(&warrant, now, ReasonCode::PolicyDigestMismatch);
        }
        let action = EnforcementAction::Freeze;
        let message = match warrant.signature_message(&action) {
            Ok(message) => message,
            Err(_) => return self.early_outcome(&warrant, now, ReasonCode::PolicySchemaInvalid),
        };
        let domain = format!(
            "{}/{}/{}",
            warrant.common.domain_binding.environment_id,
            warrant.common.domain_binding.service_id,
            warrant.common.domain_binding.adapter_id
        );
        if let Err(error) = verify_authorization(
            identities,
            &warrant.common,
            approvals,
            &message,
            &domain,
            now,
            &mut self.nonces,
        ) {
            return self.early_outcome(&warrant, now, reason_from_credential(&error));
        }

        let snapshot = match self.adapter.inspect(&warrant.common.resource) {
            Ok(snapshot) => snapshot,
            Err(error) => return self.early_outcome(&warrant, now, reason_from_adapter(&error)),
        };
        let command = EnforcementCommand {
            command_id: format!("{}:activate", warrant_id),
            warrant_id: warrant_id.clone(),
            resource: warrant.common.resource.clone(),
            action,
            amount: warrant.freeze_rule.amount,
            effective_at: now,
            expires_at: warrant.freeze_rule.expires_at,
            expected_version: snapshot.version,
        };
        let descriptor = TransitionDescriptor {
            actor: "verified-multi-authority".to_owned(),
            asset: WarrantState::Authorized,
            context: WarrantContext { risk },
            operation: command,
        };
        let manifest = self.adapter.manifest();
        let mut hooks = ActivationHooks {
            adapter: &mut self.adapter,
            warrant: &warrant,
            state: WarrantState::Authorized,
            receipt: None,
        };
        let transition = execute_transition_with_risk(&mut hooks, &descriptor, risk_policy);
        let reason_code = reason_from_outcome(&transition.outcome);
        let receipt = hooks.receipt.clone();
        let state = if matches!(transition.outcome, TransitionOutcome::Commit) {
            WarrantState::Active
        } else if matches!(transition.outcome, TransitionOutcome::Quarantine(_)) {
            WarrantState::Suspended
        } else if matches!(transition.outcome, TransitionOutcome::Reject(_)) {
            WarrantState::Rejected
        } else {
            WarrantState::Failed
        };
        let position =
            matches!(transition.outcome, TransitionOutcome::Commit).then(|| FreezePosition {
                warrant_id: warrant_id.clone(),
                asset_id: warrant.asset_scope.asset_id.clone(),
                reserved_amount: warrant.freeze_rule.amount,
                executed_amount: 0,
                released_amount: 0,
                effective_from: warrant.freeze_rule.effective_from,
                expires_at: warrant.freeze_rule.expires_at,
                status: FreezeStatus::Reserved,
            });
        let candidate_digest = receipt.as_ref().map_or_else(
            || snapshot.state_digest.clone(),
            |value| value.after_digest.clone(),
        );
        self.records.insert(
            warrant_id.clone(),
            WarrantRecord {
                warrant: warrant.clone(),
                state,
                position,
                last_receipt: receipt.clone(),
            },
        );
        let evidence = evidence_for(
            &warrant,
            now,
            &manifest,
            &snapshot.state_digest,
            &candidate_digest,
            receipt.as_ref().map(|value| value.after_digest.clone()),
            &transition.outcome,
            reason_code,
            &self.run_id,
            &self.source_commit,
        );
        let _ = self.audit.append(evidence);
        EnforcementResult {
            outcome: transition.outcome,
            reason_code,
            receipt,
        }
    }

    fn early_outcome(
        &mut self,
        warrant: &AssetFreezingWarrant,
        now: Timestamp,
        reason_code: ReasonCode,
    ) -> EnforcementResult {
        let manifest = self.adapter.manifest();
        let outcome =
            TransitionOutcome::Reject(RejectReason::InvalidInput(format!("{reason_code:?}")));
        let evidence = evidence_for(
            warrant,
            now,
            &manifest,
            "unavailable",
            "unavailable",
            None,
            &outcome,
            reason_code,
            &self.run_id,
            &self.source_commit,
        );
        let _ = self.audit.append(evidence);
        EnforcementResult {
            outcome,
            reason_code,
            receipt: None,
        }
    }
}

#[derive(Debug, Clone)]
struct WarrantContext {
    risk: Option<RiskEvidence>,
}

struct ActivationHooks<'a, A> {
    adapter: &'a mut A,
    warrant: &'a AssetFreezingWarrant,
    state: WarrantState,
    receipt: Option<ExecutionReceipt>,
}

impl<A: EnforcementAdapter>
    RiskAwareAacoHooks<String, WarrantState, WarrantContext, EnforcementCommand>
    for ActivationHooks<'_, A>
{
    type Candidate = PreparedChange;
    type State = WarrantState;

    fn authorize(
        &self,
        descriptor: &TransitionDescriptor<String, WarrantState, WarrantContext, EnforcementCommand>,
    ) -> Result<(), TransitionFailure> {
        if let Some(risk) = &descriptor.context.risk {
            return Err(TransitionFailure::Risk(risk.clone()));
        }
        if descriptor.actor != "verified-multi-authority" {
            return Err(TransitionFailure::Policy(RejectReason::DisallowedActor));
        }
        Ok(())
    }

    fn validate(
        &self,
        descriptor: &TransitionDescriptor<String, WarrantState, WarrantContext, EnforcementCommand>,
    ) -> Result<(), TransitionFailure> {
        if !descriptor.asset.allows(WarrantState::Active) {
            return Err(TransitionFailure::Policy(RejectReason::InvariantViolation(
                "WARRANT_INVALID_TRANSITION".into(),
            )));
        }
        let manifest = self.adapter.manifest();
        require_capabilities(
            &manifest,
            &descriptor.operation.resource.kind,
            &descriptor.operation.action,
        )
        .map_err(adapter_failure)?;
        if manifest.adapter_id.0 != self.warrant.common.domain_binding.adapter_id {
            return Err(TransitionFailure::Policy(RejectReason::InvalidInput(
                "AUTH_DOMAIN_MISMATCH".into(),
            )));
        }
        Ok(())
    }

    fn mutate_candidate(
        &mut self,
        descriptor: &TransitionDescriptor<String, WarrantState, WarrantContext, EnforcementCommand>,
    ) -> Result<Self::Candidate, TransitionFailure> {
        self.adapter
            .prepare(&descriptor.operation)
            .map_err(adapter_failure)
    }

    fn reconcile(&self, candidate: &Self::Candidate) -> Result<(), TransitionFailure> {
        if candidate.command.amount > self.warrant.asset_scope.maximum_amount
            || candidate.candidate.active_reserved_amount < candidate.before.active_reserved_amount
        {
            return Err(TransitionFailure::Policy(RejectReason::InvariantViolation(
                "WARRANT_AMOUNT_EXCEEDED".into(),
            )));
        }
        Ok(())
    }

    fn commit(&mut self, candidate: Self::Candidate) -> Result<(), TransitionFailure> {
        self.receipt = Some(self.adapter.commit(candidate).map_err(adapter_failure)?);
        self.state = WarrantState::Active;
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.state
    }
}

fn adapter_failure(error: AdapterError) -> TransitionFailure {
    match error {
        AdapterError::UnsupportedCapability
        | AdapterError::StaleSnapshot
        | AdapterError::DuplicateCommand
        | AdapterError::InvalidAmount
        | AdapterError::NonAuthoritativeBalance
        | AdapterError::ResourceNotFound => {
            TransitionFailure::Policy(RejectReason::InvalidInput(error.to_string()))
        }
        AdapterError::PrepareFailed | AdapterError::CommitFailed => {
            TransitionFailure::Internal(AbortReason::InternalFailure(error.to_string()))
        }
    }
}

fn reason_from_credential(error: &CredentialError) -> ReasonCode {
    match error {
        CredentialError::MissingRequiredRole => ReasonCode::AuthMissingRequiredRole,
        CredentialError::InvalidSignature => ReasonCode::AuthInvalidSignature,
        CredentialError::ReplayNonce => ReasonCode::AuthReplayNonce,
        CredentialError::DomainMismatch => ReasonCode::AuthDomainMismatch,
        CredentialError::RevokedCredential | CredentialError::ExpiredCredential => {
            ReasonCode::AuthRevokedCredential
        }
        _ => ReasonCode::AuthInvalidSignature,
    }
}

fn reason_from_adapter(error: &AdapterError) -> ReasonCode {
    match error {
        AdapterError::UnsupportedCapability | AdapterError::NonAuthoritativeBalance => {
            ReasonCode::AdapterUnsupportedCapability
        }
        AdapterError::StaleSnapshot => ReasonCode::AdapterStaleSnapshot,
        AdapterError::PrepareFailed => ReasonCode::AdapterPrepareFailed,
        AdapterError::CommitFailed => ReasonCode::AdapterCommitFailed,
        _ => ReasonCode::SystemAborted,
    }
}

fn reason_from_outcome(outcome: &TransitionOutcome) -> ReasonCode {
    match outcome {
        TransitionOutcome::Commit => ReasonCode::Commit,
        TransitionOutcome::Quarantine(_) => ReasonCode::RiskQuarantined,
        TransitionOutcome::Abort(_) => ReasonCode::AdapterCommitFailed,
        TransitionOutcome::Reject(reason) => {
            let text = format!("{reason:?}");
            if text.contains("UNSUPPORTED") {
                ReasonCode::AdapterUnsupportedCapability
            } else if text.contains("STALE") {
                ReasonCode::AdapterStaleSnapshot
            } else if text.contains("AMOUNT") {
                ReasonCode::WarrantAmountExceeded
            } else {
                ReasonCode::SystemAborted
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn evidence_for(
    warrant: &AssetFreezingWarrant,
    timestamp: Timestamp,
    manifest: &hete_adapter_api::AdapterManifest,
    before_digest: &str,
    candidate_digest: &str,
    after_digest: Option<String>,
    outcome: &TransitionOutcome,
    reason_code: ReasonCode,
    run_id: &str,
    source_commit: &str,
) -> EnforcementEvidence {
    EnforcementEvidence {
        audit_id: format!("audit:{}:{}", warrant.warrant_reference.0, timestamp),
        policy_digest: warrant.common.policy_digest.clone(),
        transition_id: format!("{}:activate", warrant.warrant_reference.0),
        actor_refs: Vec::new(),
        authority_roles: warrant
            .common
            .authorization_policy
            .requirements
            .iter()
            .map(|requirement| requirement.role.clone())
            .collect(),
        adapter_id: manifest.adapter_id.clone(),
        adapter_version: manifest.version.clone(),
        before_digest: before_digest.into(),
        candidate_digest: candidate_digest.into(),
        after_digest,
        outcome: outcome.label().into(),
        reason_code,
        timestamp,
        run_id: run_id.into(),
        source_commit: source_commit.into(),
        previous_hash: None,
        record_hash: String::new(),
    }
}

fn digest<T: Serialize>(value: &T) -> Result<String, WarrantError> {
    let bytes = poa_protocol::canonicalize_value(value).map_err(|_| WarrantError::PolicyInvalid)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCommitment {
    pub policy_digest: PolicyDigest,
    pub nonce: Nonce,
    pub expires_at: Timestamp,
}

#[derive(Debug, Default)]
pub struct CommitmentRegistry(BTreeSet<(PolicyDigest, Nonce)>);

impl CommitmentRegistry {
    pub fn register(
        &mut self,
        commitment: &PolicyCommitment,
        now: Timestamp,
    ) -> Result<(), WarrantError> {
        if now >= commitment.expires_at
            || !self
                .0
                .insert((commitment.policy_digest.clone(), commitment.nonce.clone()))
        {
            return Err(WarrantError::DuplicateExecution);
        }
        Ok(())
    }

    pub fn reveal(
        &mut self,
        policy: &MachinePolicyObject,
        now: Timestamp,
    ) -> Result<(), WarrantError> {
        let key = (policy.policy_digest.clone(), policy.nonce.clone());
        if now >= policy.validity.expires_at || !self.0.remove(&key) {
            return Err(WarrantError::PolicyInvalid);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SealedPolicyEnvelope {
    pub envelope_id: String,
    pub ciphertext: Vec<u8>,
    pub expires_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngressReceipt {
    pub envelope_id: String,
    pub accepted_at: Timestamp,
    pub assurance: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IngressError {
    #[error("INGRESS_EXPIRED")]
    Expired,
    #[error("INGRESS_DUPLICATE")]
    Duplicate,
}

pub trait ConfidentialIngress {
    fn submit_sealed(
        &mut self,
        envelope: SealedPolicyEnvelope,
    ) -> Result<IngressReceipt, IngressError>;
}

#[derive(Debug, Default)]
pub struct LocalSealedIngress {
    pub now: Timestamp,
    accepted: BTreeSet<String>,
}

impl ConfidentialIngress for LocalSealedIngress {
    fn submit_sealed(
        &mut self,
        envelope: SealedPolicyEnvelope,
    ) -> Result<IngressReceipt, IngressError> {
        if self.now >= envelope.expires_at {
            return Err(IngressError::Expired);
        }
        if !self.accepted.insert(envelope.envelope_id.clone()) {
            return Err(IngressError::Duplicate);
        }
        Ok(IngressReceipt {
            envelope_id: envelope.envelope_id,
            accepted_at: self.now,
            assurance: "local-sealed-envelope-simulation-only".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn forbidden_terminal_transitions_fail() {
        assert!(!WarrantState::Rejected.allows(WarrantState::Active));
        assert!(!WarrantState::Expired.allows(WarrantState::Active));
        assert!(!WarrantState::Revoked.allows(WarrantState::Active));
        assert!(!WarrantState::Released.allows(WarrantState::PartiallyExecuted));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]
        #[test]
        fn amount_conservation(
            reserved in 0_u128..1_000_000,
            executed in 0_u128..1_000_000,
            released in 0_u128..1_000_000,
        ) {
            let position = FreezePosition {
                warrant_id: "w".into(), asset_id: "a".into(), reserved_amount: reserved,
                executed_amount: executed, released_amount: released, effective_from: 0,
                expires_at: 1, status: FreezeStatus::Reserved,
            };
            let valid = executed.saturating_add(released) <= reserved;
            prop_assert_eq!(position.validate(reserved).is_ok(), valid);
        }
    }

    #[test]
    fn public_commit_does_not_contain_target() {
        let commitment = PolicyCommitment {
            policy_digest: PolicyDigest("sha256:abc".into()),
            nonce: Nonce("n".into()),
            expires_at: 10,
        };
        let text = serde_json::to_string(&commitment).unwrap();
        assert!(!text.contains("target"));
    }

    #[test]
    fn audit_chain_detects_tampering() {
        let mut chain = AuditChain::default();
        let evidence = EnforcementEvidence {
            audit_id: "audit-1".into(),
            policy_digest: PolicyDigest("sha256:fixture".into()),
            transition_id: "transition-1".into(),
            actor_refs: vec![],
            authority_roles: vec![AuthorityRole::Auditor],
            adapter_id: AdapterId("adapter".into()),
            adapter_version: "1".into(),
            before_digest: "before".into(),
            candidate_digest: "candidate".into(),
            after_digest: None,
            outcome: "reject".into(),
            reason_code: ReasonCode::SystemAborted,
            timestamp: 1,
            run_id: "run".into(),
            source_commit: "commit".into(),
            previous_hash: None,
            record_hash: String::new(),
        };
        chain.append(evidence).unwrap();
        assert!(chain.verify());
        chain.0[0].outcome = "commit".into();
        assert!(!chain.verify());
    }

    #[test]
    fn lifecycle_amount_updates_cannot_resurrect_terminal_state() {
        let mut record = WarrantRecord {
            warrant: unsafe_fixture_warrant(),
            state: WarrantState::Active,
            position: Some(FreezePosition {
                warrant_id: "w1".into(),
                asset_id: "a1".into(),
                reserved_amount: 100,
                executed_amount: 0,
                released_amount: 0,
                effective_from: 1,
                expires_at: 10,
                status: FreezeStatus::Reserved,
            }),
            last_receipt: None,
        };
        record.record_execution(40).unwrap();
        assert_eq!(record.state, WarrantState::PartiallyExecuted);
        record.record_release(0).unwrap();
        assert_eq!(record.state, WarrantState::Released);
        assert_eq!(
            record.record_execution(1),
            Err(WarrantError::InvalidTransition)
        );
    }

    fn unsafe_fixture_warrant() -> AssetFreezingWarrant {
        use hete_identity::Did;
        use hete_policy::{
            ActionConstraint, AuthorizationPolicy, PolicyId, PseudonymousTargetRef, ResourceKind,
            ResourceRef, RevocationRule, RoleRequirement, SubjectRef, ValidityWindow,
        };
        let target = PseudonymousTargetRef::derive("test", "subject", "a1", "w1", b"salt");
        let authority = AuthorityRef {
            did: Did("did:key:authority".into()),
            role: AuthorityRole::JudicialIssuer,
        };
        let mut common = MachinePolicyObject {
            policy_id: PolicyId("p1".into()),
            policy_type: "electronic_warrant".into(),
            version: "1".into(),
            issuer: authority.clone(),
            authorization_policy: AuthorizationPolicy {
                requirements: vec![RoleRequirement {
                    role: AuthorityRole::JudicialIssuer,
                    minimum_signatures: 1,
                    sequence: 1,
                }],
                threshold: 1,
                mutually_exclusive_roles: vec![],
                sequential: false,
            },
            subject: SubjectRef {
                target: target.clone(),
            },
            resource: ResourceRef {
                resource_id: "a1".into(),
                kind: ResourceKind::Account,
                target,
            },
            permitted_actions: vec![ActionConstraint {
                action: EnforcementAction::Freeze,
                maximum_amount: Some(100),
            }],
            validity: ValidityWindow {
                not_before: 1,
                expires_at: 10,
                maximum_duration: 9,
            },
            conditions: vec![],
            obligations: vec![],
            revocation: RevocationRule {
                revocable: true,
                required_role: AuthorityRole::JudicialIssuer,
            },
            evidence_refs: vec![],
            credential_refs: vec![],
            nonce: Nonce("n1".into()),
            domain_binding: hete_policy::DomainBinding {
                environment_id: "test".into(),
                service_id: "svc".into(),
                adapter_id: "adapter".into(),
            },
            policy_digest: PolicyDigest(String::new()),
        };
        common.seal().unwrap();
        AssetFreezingWarrant {
            common,
            case_reference: PseudonymousCaseRef("case".into()),
            warrant_reference: WarrantRef("w1".into()),
            jurisdiction: JurisdictionRef("fixture".into()),
            asset_scope: AssetScope {
                asset_id: "a1".into(),
                maximum_amount: 100,
                inflow_rule: InflowRule::ExistingBalanceOnly,
            },
            freeze_rule: FreezeRule {
                amount: 100,
                effective_from: 1,
                expires_at: 10,
            },
            execution_rule: ExecutionRule {
                partial_execution_allowed: true,
                destination_ref: None,
            },
            requesting_authority: authority.clone(),
            supervisory_authority: None,
            judicial_issuer: authority,
            review_rule: ReviewRule {
                review_at: 5,
                reviewer_role: AuthorityRole::Auditor,
            },
            appeal_or_hold_rule: None,
        }
    }
}
