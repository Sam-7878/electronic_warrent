//! Second reference domain: bounded delegation of AI-agent tool invocations.
//!
//! This crate demonstrates reuse of `MachinePolicyObject`, `poa-core` typed
//! descriptors, and `EnforcementAdapter` without adding domain branches to core.

use hete_adapter_api::{AdapterError, EnforcementAdapter, EnforcementCommand, ExecutionReceipt};
use hete_identity::Timestamp;
use hete_policy::{EnforcementAction, MachinePolicyObject};
use poa_core::TransitionDescriptor;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const POLICY_TYPE: &str = "hete.ai_agent_tool_delegation.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentToolDelegation {
    pub common: MachinePolicyObject,
    pub agent_id: String,
    pub tool_id: String,
    pub allowed_scopes: Vec<String>,
    pub maximum_calls: u32,
    pub human_confirmation_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationState {
    Draft,
    Active,
    Revoked,
    Expired,
    Exhausted,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DelegationError {
    #[error("DELEGATION_POLICY_INVALID")]
    InvalidPolicy,
    #[error("DELEGATION_SCOPE_DENIED")]
    ScopeDenied,
    #[error("DELEGATION_HUMAN_CONFIRMATION_REQUIRED")]
    HumanConfirmationRequired,
    #[error("DELEGATION_NOT_ACTIVE")]
    NotActive,
    #[error("DELEGATION_EXHAUSTED")]
    Exhausted,
    #[error("DELEGATION_ADAPTER_FAILED")]
    AdapterFailed,
}

impl AgentToolDelegation {
    pub fn validate(&self, now: Timestamp) -> Result<(), DelegationError> {
        self.common
            .validate(now)
            .map_err(|_| DelegationError::InvalidPolicy)?;
        let permitted = self.common.permitted_actions.iter().any(|constraint| {
            constraint.action == EnforcementAction::Custom("invoke_agent_tool".into())
        });
        if self.common.policy_type != POLICY_TYPE
            || self.agent_id.is_empty()
            || self.tool_id.is_empty()
            || self.allowed_scopes.is_empty()
            || self.maximum_calls == 0
            || !permitted
        {
            return Err(DelegationError::InvalidPolicy);
        }
        Ok(())
    }
}

pub struct DelegationRuntime {
    delegation: AgentToolDelegation,
    state: DelegationState,
    calls: u32,
}

impl DelegationRuntime {
    pub fn new(delegation: AgentToolDelegation, now: Timestamp) -> Result<Self, DelegationError> {
        delegation.validate(now)?;
        Ok(Self {
            delegation,
            state: DelegationState::Draft,
            calls: 0,
        })
    }

    pub fn activate(&mut self) -> Result<(), DelegationError> {
        if self.state != DelegationState::Draft {
            return Err(DelegationError::NotActive);
        }
        self.state = DelegationState::Active;
        Ok(())
    }

    pub fn revoke(&mut self) {
        self.state = DelegationState::Revoked;
    }

    pub fn state(&self) -> DelegationState {
        self.state
    }
    pub fn calls(&self) -> u32 {
        self.calls
    }

    pub fn descriptor<'a>(
        &'a self,
        scope: &'a str,
    ) -> TransitionDescriptor<&'a str, &'a str, &'a str, &'a str> {
        TransitionDescriptor {
            actor: &self.delegation.agent_id,
            asset: &self.delegation.tool_id,
            context: scope,
            operation: "invoke_agent_tool",
        }
    }

    pub fn invoke<A: EnforcementAdapter>(
        &mut self,
        adapter: &mut A,
        scope: &str,
        human_confirmed: bool,
        now: Timestamp,
    ) -> Result<ExecutionReceipt, DelegationError> {
        if now >= self.delegation.common.validity.expires_at {
            self.state = DelegationState::Expired;
            return Err(DelegationError::NotActive);
        }
        if self.state != DelegationState::Active {
            return Err(DelegationError::NotActive);
        }
        if self.calls >= self.delegation.maximum_calls {
            self.state = DelegationState::Exhausted;
            return Err(DelegationError::Exhausted);
        }
        if !self
            .delegation
            .allowed_scopes
            .iter()
            .any(|allowed| allowed == scope)
        {
            return Err(DelegationError::ScopeDenied);
        }
        if self.delegation.human_confirmation_required && !human_confirmed {
            return Err(DelegationError::HumanConfirmationRequired);
        }
        let snapshot = adapter
            .inspect(&self.delegation.common.resource)
            .map_err(|_| DelegationError::AdapterFailed)?;
        let command = EnforcementCommand {
            command_id: format!("{}:{}", self.delegation.common.policy_id.0, self.calls),
            warrant_id: self.delegation.common.policy_id.0.clone(),
            resource: self.delegation.common.resource.clone(),
            action: EnforcementAction::Custom("invoke_agent_tool".into()),
            amount: 1,
            effective_at: now,
            expires_at: self.delegation.common.validity.expires_at,
            expected_version: snapshot.version,
        };
        let prepared = adapter.prepare(&command).map_err(map_adapter)?;
        let receipt = adapter.commit(prepared).map_err(map_adapter)?;
        self.calls += 1;
        if self.calls == self.delegation.maximum_calls {
            self.state = DelegationState::Exhausted;
        }
        Ok(receipt)
    }
}

fn map_adapter(_: AdapterError) -> DelegationError {
    DelegationError::AdapterFailed
}

#[cfg(test)]
mod tests {
    use super::*;
    use hete_adapter_api::{AdapterId, AdapterManifest, PreparedChange, ResourceSnapshot};
    use hete_identity::{AuthorityRole, Did};
    use hete_policy::{
        ActionConstraint, AuthorityRef, AuthorizationPolicy, DomainBinding, Nonce, PolicyDigest,
        PolicyId, PseudonymousTargetRef, ResourceKind, ResourceRef, RevocationRule,
        RoleRequirement, SubjectRef, ValidityWindow,
    };

    struct ToolAdapter {
        resource: ResourceRef,
        version: u64,
    }

    impl EnforcementAdapter for ToolAdapter {
        fn manifest(&self) -> AdapterManifest {
            AdapterManifest {
                adapter_id: AdapterId("tool-observer".into()),
                version: "1".into(),
                supported_resources: vec![ResourceKind::Device],
                supported_actions: vec![EnforcementAction::Custom("invoke_agent_tool".into())],
                supports_atomic_prepare_commit: true,
                supports_amount_bounded_freeze: false,
                supports_expiration: true,
                supports_revocation: true,
                authoritative_balance: true,
                assurance: "test".into(),
            }
        }
        fn inspect(&self, _: &ResourceRef) -> Result<ResourceSnapshot, AdapterError> {
            Ok(ResourceSnapshot {
                resource: self.resource.clone(),
                balance: Some(10),
                active_reserved_amount: 0,
                pending_execution_amount: 0,
                version: self.version,
                state_digest: format!("v{}", self.version),
            })
        }
        fn prepare(
            &mut self,
            command: &EnforcementCommand,
        ) -> Result<PreparedChange, AdapterError> {
            if command.expected_version != self.version {
                return Err(AdapterError::StaleSnapshot);
            }
            let before = self.inspect(&command.resource)?;
            let mut candidate = before.clone();
            candidate.version += 1;
            candidate.state_digest = format!("v{}", candidate.version);
            Ok(PreparedChange {
                command: command.clone(),
                before,
                candidate,
                preparation_digest: "p".into(),
            })
        }
        fn commit(&mut self, prepared: PreparedChange) -> Result<ExecutionReceipt, AdapterError> {
            if prepared.before.version != self.version {
                return Err(AdapterError::StaleSnapshot);
            }
            self.version += 1;
            Ok(ExecutionReceipt {
                receipt_id: format!("r{}", self.version),
                command_id: prepared.command.command_id,
                adapter_id: AdapterId("tool-observer".into()),
                adapter_version: "1".into(),
                before_digest: prepared.before.state_digest,
                after_digest: prepared.candidate.state_digest,
                committed_at: prepared.command.effective_at,
                dry_run: true,
            })
        }
        fn rollback(&mut self, _: PreparedChange) -> Result<(), AdapterError> {
            Ok(())
        }
    }

    fn fixture() -> AgentToolDelegation {
        let target = PseudonymousTargetRef::derive("agent", "a1", "tool-1", "p1", b"salt");
        let authority = AuthorityRef {
            did: Did("did:example:owner".into()),
            role: AuthorityRole::Requester,
        };
        let resource = ResourceRef {
            resource_id: "tool-1".into(),
            kind: ResourceKind::Device,
            target: target.clone(),
        };
        let mut common = MachinePolicyObject {
            policy_id: PolicyId("delegation-1".into()),
            policy_type: POLICY_TYPE.into(),
            version: "1".into(),
            issuer: authority.clone(),
            authorization_policy: AuthorizationPolicy {
                requirements: vec![RoleRequirement {
                    role: AuthorityRole::Requester,
                    minimum_signatures: 1,
                    sequence: 0,
                }],
                threshold: 1,
                mutually_exclusive_roles: vec![],
                sequential: false,
            },
            subject: SubjectRef { target },
            resource,
            permitted_actions: vec![ActionConstraint {
                action: EnforcementAction::Custom("invoke_agent_tool".into()),
                maximum_amount: Some(1),
            }],
            validity: ValidityWindow {
                not_before: 1,
                expires_at: 100,
                maximum_duration: 100,
            },
            conditions: vec![],
            obligations: vec![],
            revocation: RevocationRule {
                revocable: true,
                required_role: AuthorityRole::Requester,
            },
            evidence_refs: vec![],
            credential_refs: vec![],
            nonce: Nonce("n1".into()),
            domain_binding: DomainBinding {
                environment_id: "test".into(),
                service_id: "agent".into(),
                adapter_id: "tool-observer".into(),
            },
            policy_digest: PolicyDigest(String::new()),
        };
        common.seal().unwrap();
        AgentToolDelegation {
            common,
            agent_id: "agent-1".into(),
            tool_id: "tool-1".into(),
            allowed_scopes: vec!["read".into()],
            maximum_calls: 2,
            human_confirmation_required: true,
        }
    }

    fn runtime() -> (DelegationRuntime, ToolAdapter) {
        let delegation = fixture();
        let resource = delegation.common.resource.clone();
        (
            DelegationRuntime::new(delegation, 10).unwrap(),
            ToolAdapter {
                resource,
                version: 0,
            },
        )
    }

    #[test]
    fn valid_policy_activates() {
        let (mut r, _) = runtime();
        r.activate().unwrap();
        assert_eq!(r.state(), DelegationState::Active);
    }
    #[test]
    fn wrong_policy_type_fails_closed() {
        let mut d = fixture();
        d.common.policy_type = "wrong".into();
        d.common.seal().unwrap();
        assert_eq!(
            DelegationRuntime::new(d, 10).err(),
            Some(DelegationError::InvalidPolicy)
        );
    }
    #[test]
    fn scope_expansion_is_rejected() {
        let (mut r, mut a) = runtime();
        r.activate().unwrap();
        assert_eq!(
            r.invoke(&mut a, "write", true, 10),
            Err(DelegationError::ScopeDenied)
        );
    }
    #[test]
    fn human_confirmation_cannot_be_bypassed() {
        let (mut r, mut a) = runtime();
        r.activate().unwrap();
        assert_eq!(
            r.invoke(&mut a, "read", false, 10),
            Err(DelegationError::HumanConfirmationRequired)
        );
    }
    #[test]
    fn valid_invocation_commits_receipt() {
        let (mut r, mut a) = runtime();
        r.activate().unwrap();
        assert!(r.invoke(&mut a, "read", true, 10).unwrap().dry_run);
        assert_eq!(r.calls(), 1);
    }
    #[test]
    fn call_limit_is_enforced() {
        let (mut r, mut a) = runtime();
        r.activate().unwrap();
        r.invoke(&mut a, "read", true, 10).unwrap();
        r.invoke(&mut a, "read", true, 10).unwrap();
        assert_eq!(r.state(), DelegationState::Exhausted);
        assert_eq!(
            r.invoke(&mut a, "read", true, 10),
            Err(DelegationError::NotActive)
        );
    }
    #[test]
    fn revocation_blocks_invocation() {
        let (mut r, mut a) = runtime();
        r.activate().unwrap();
        r.revoke();
        assert_eq!(
            r.invoke(&mut a, "read", true, 10),
            Err(DelegationError::NotActive)
        );
    }
    #[test]
    fn expiry_blocks_invocation() {
        let (mut r, mut a) = runtime();
        r.activate().unwrap();
        assert_eq!(
            r.invoke(&mut a, "read", true, 100),
            Err(DelegationError::NotActive)
        );
        assert_eq!(r.state(), DelegationState::Expired);
    }
    #[test]
    fn descriptor_is_domain_neutral() {
        let (r, _) = runtime();
        let d = r.descriptor("read");
        assert_eq!(d.operation, "invoke_agent_tool");
    }
}
