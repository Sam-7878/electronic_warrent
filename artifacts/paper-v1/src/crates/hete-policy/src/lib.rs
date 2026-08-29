//! Domain-neutral machine-verifiable policy objects.
//!
//! Deserialization rejects unknown fields. Digest verification and semantic
//! validation are explicit operations; callers must perform both before using a
//! policy for authorization.

use std::collections::{BTreeMap, BTreeSet};

use hete_identity::{AuthorityRole, Did, Timestamp};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PolicyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Nonce(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PolicyDigest(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityRef {
    pub did: Did,
    pub role: AuthorityRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleRequirement {
    pub role: AuthorityRole,
    pub minimum_signatures: u16,
    pub sequence: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationPolicy {
    pub requirements: Vec<RoleRequirement>,
    pub threshold: u16,
    #[serde(default)]
    pub mutually_exclusive_roles: Vec<Vec<AuthorityRole>>,
    #[serde(default)]
    pub sequential: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetRefScheme {
    Sha256Salted,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PseudonymousTargetRef {
    pub scheme: TargetRefScheme,
    pub digest: [u8; 32],
    pub salt_id: Option<String>,
    pub epoch: Option<u64>,
}

impl PseudonymousTargetRef {
    pub fn derive(
        domain_separator: &str,
        subject_identifier: &str,
        resource_id: &str,
        warrant_id: &str,
        random_salt: &[u8],
    ) -> Self {
        let normalized = subject_identifier.trim().to_lowercase();
        let mut hasher = Sha256::new();
        for part in [
            domain_separator.as_bytes(),
            normalized.as_bytes(),
            resource_id.as_bytes(),
            warrant_id.as_bytes(),
            random_salt,
        ] {
            hasher.update((part.len() as u64).to_be_bytes());
            hasher.update(part);
        }
        Self {
            scheme: TargetRefScheme::Sha256Salted,
            digest: hasher.finalize().into(),
            salt_id: None,
            epoch: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectRef {
    pub target: PseudonymousTargetRef,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Account,
    Wallet,
    Vault,
    Token,
    Device,
    Data,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRef {
    pub resource_id: String,
    pub kind: ResourceKind,
    pub target: PseudonymousTargetRef,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementAction {
    Freeze,
    Release,
    Execute,
    Inspect,
    Suspend,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionConstraint {
    pub action: EnforcementAction,
    pub maximum_amount: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidityWindow {
    pub not_before: Timestamp,
    pub expires_at: Timestamp,
    pub maximum_duration: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyCondition {
    pub name: String,
    pub value: String,
    pub critical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyObligation {
    pub name: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevocationRule {
    pub revocable: bool,
    pub required_role: AuthorityRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    pub id: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialRef {
    pub id: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainBinding {
    pub environment_id: String,
    pub service_id: String,
    pub adapter_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MachinePolicyObject {
    pub policy_id: PolicyId,
    pub policy_type: String,
    pub version: String,
    pub issuer: AuthorityRef,
    pub authorization_policy: AuthorizationPolicy,
    pub subject: SubjectRef,
    pub resource: ResourceRef,
    pub permitted_actions: Vec<ActionConstraint>,
    pub validity: ValidityWindow,
    #[serde(default)]
    pub conditions: Vec<PolicyCondition>,
    #[serde(default)]
    pub obligations: Vec<PolicyObligation>,
    pub revocation: RevocationRule,
    #[serde(default)]
    pub evidence_refs: Vec<EvidenceRef>,
    #[serde(default)]
    pub credential_refs: Vec<CredentialRef>,
    pub nonce: Nonce,
    pub domain_binding: DomainBinding,
    pub policy_digest: PolicyDigest,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PolicyError {
    #[error("POLICY_SCHEMA_INVALID")]
    SchemaInvalid,
    #[error("POLICY_DIGEST_MISMATCH")]
    DigestMismatch,
    #[error("POLICY_PRIVILEGE_EXPANSION")]
    PrivilegeExpansion,
    #[error("POLICY_TIME_INVALID")]
    InvalidTime,
    #[error("POLICY_AUTHORIZATION_INVALID")]
    InvalidAuthorization,
    #[error("POLICY_CANONICALIZATION_FAILED")]
    Canonicalization,
}

impl MachinePolicyObject {
    /// Compute the digest over every policy field except `policy_digest` itself.
    pub fn compute_digest(&self) -> Result<PolicyDigest, PolicyError> {
        let mut value = serde_json::to_value(self).map_err(|_| PolicyError::Canonicalization)?;
        value
            .as_object_mut()
            .ok_or(PolicyError::Canonicalization)?
            .remove("policy_digest");
        let bytes =
            poa_protocol::canonicalize_value(&value).map_err(|_| PolicyError::Canonicalization)?;
        Ok(PolicyDigest(format!(
            "sha256:{}",
            hex::encode(Sha256::digest(bytes))
        )))
    }

    pub fn seal(&mut self) -> Result<(), PolicyError> {
        self.policy_digest = self.compute_digest()?;
        Ok(())
    }

    /// Validate the time, authorization, and embedded digest constraints.
    pub fn validate(&self, now: Timestamp) -> Result<(), PolicyError> {
        if self.validity.not_before >= self.validity.expires_at
            || self.validity.expires_at - self.validity.not_before > self.validity.maximum_duration
            || now < self.validity.not_before
            || now >= self.validity.expires_at
        {
            return Err(PolicyError::InvalidTime);
        }
        let requirements: u16 = self
            .authorization_policy
            .requirements
            .iter()
            .map(|requirement| requirement.minimum_signatures)
            .sum();
        if self.authorization_policy.threshold == 0
            || self.authorization_policy.threshold > requirements
            || self
                .authorization_policy
                .requirements
                .iter()
                .any(|requirement| requirement.minimum_signatures == 0)
        {
            return Err(PolicyError::InvalidAuthorization);
        }
        if self.compute_digest()? != self.policy_digest {
            return Err(PolicyError::DigestMismatch);
        }
        Ok(())
    }
}

/// Reject a child policy that expands authority, actions, duration, or amount.
pub fn ensure_no_privilege_expansion(
    parent: &MachinePolicyObject,
    child: &MachinePolicyObject,
) -> Result<(), PolicyError> {
    let parent_roles: BTreeSet<_> = parent
        .authorization_policy
        .requirements
        .iter()
        .map(|requirement| &requirement.role)
        .collect();
    let child_roles: BTreeSet<_> = child
        .authorization_policy
        .requirements
        .iter()
        .map(|requirement| &requirement.role)
        .collect();
    if !child_roles.is_subset(&parent_roles)
        || child.authorization_policy.threshold < parent.authorization_policy.threshold
        || child.validity.maximum_duration > parent.validity.maximum_duration
    {
        return Err(PolicyError::PrivilegeExpansion);
    }
    let parent_actions: BTreeMap<_, _> = parent
        .permitted_actions
        .iter()
        .map(|constraint| (&constraint.action, constraint.maximum_amount))
        .collect();
    for constraint in &child.permitted_actions {
        let Some(parent_maximum) = parent_actions.get(&constraint.action) else {
            return Err(PolicyError::PrivilegeExpansion);
        };
        match (parent_maximum, constraint.maximum_amount) {
            (Some(parent), Some(child)) if child <= *parent => {}
            (None, _) => {}
            _ => return Err(PolicyError::PrivilegeExpansion),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> MachinePolicyObject {
        let target =
            PseudonymousTargetRef::derive("HETE", "did:example:subject", "asset", "w1", b"salt");
        let mut policy = MachinePolicyObject {
            policy_id: PolicyId("p1".into()),
            policy_type: "reference_policy".into(),
            version: "1".into(),
            issuer: AuthorityRef {
                did: Did("did:example:issuer".into()),
                role: AuthorityRole::Custom("issuer".into()),
            },
            authorization_policy: AuthorizationPolicy {
                requirements: vec![RoleRequirement {
                    role: AuthorityRole::Custom("reviewer".into()),
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
                resource_id: "asset".into(),
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
                required_role: AuthorityRole::Custom("reviewer".into()),
            },
            evidence_refs: vec![],
            credential_refs: vec![],
            nonce: Nonce("n1".into()),
            domain_binding: DomainBinding {
                environment_id: "test".into(),
                service_id: "svc".into(),
                adapter_id: "adapter".into(),
            },
            policy_digest: PolicyDigest(String::new()),
        };
        policy.seal().unwrap();
        policy
    }

    #[test]
    fn pol_001_digest_is_stable() {
        let policy = fixture();
        assert_eq!(policy.compute_digest().unwrap(), policy.policy_digest);
    }

    #[test]
    fn pol_002_mutation_changes_digest() {
        let mut policy = fixture();
        let original = policy.compute_digest().unwrap();
        policy.permitted_actions[0].maximum_amount = Some(101);
        assert_ne!(policy.compute_digest().unwrap(), original);
    }

    #[test]
    fn pol_003_effective_digest_is_not_parent_fragment_digest() {
        let parent = fixture();
        let mut effective = parent.clone();
        effective.validity.expires_at = 9;
        effective.validity.maximum_duration = 8;
        effective.seal().unwrap();
        assert!(ensure_no_privilege_expansion(&parent, &effective).is_ok());
        assert_ne!(parent.policy_digest, effective.policy_digest);
    }

    #[test]
    fn pol_004_privilege_expansion_rejected() {
        let parent = fixture();
        let mut child = parent.clone();
        child.permitted_actions[0].maximum_amount = Some(101);
        assert_eq!(
            ensure_no_privilege_expansion(&parent, &child),
            Err(PolicyError::PrivilegeExpansion)
        );
    }

    #[test]
    fn pol_005_unknown_field_rejected() {
        let mut value = serde_json::to_value(fixture()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("critical_extension".into(), true.into());
        assert!(serde_json::from_value::<MachinePolicyObject>(value).is_err());
    }
}
