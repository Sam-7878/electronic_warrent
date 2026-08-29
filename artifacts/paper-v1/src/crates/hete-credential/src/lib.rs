//! Minimal deterministic credential and multi-authority approval profile.
//!
//! This is not a complete W3C VC implementation. It verifies the explicit
//! Ed25519 fixture profile used by the reference evaluation and rejects unknown,
//! expired, revoked, cross-domain, or replayed inputs.

use std::collections::{BTreeMap, BTreeSet};

use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Verifier};
use hete_identity::{
    AuthorityRole, AuthorityStatus, Did, IdentityError, LocalIdentityStore, Timestamp,
    TrustRegistry,
};
use hete_policy::{AuthorizationPolicy, MachinePolicyObject, Nonce, SubjectRef};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialProof {
    pub key_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialEnvelope {
    pub id: String,
    pub issuer: Did,
    pub subject: SubjectRef,
    pub credential_type: Vec<String>,
    pub issuance_time: Timestamp,
    pub expiration_time: Option<Timestamp>,
    pub status: Option<String>,
    pub claims: serde_json::Value,
    pub proof: CredentialProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationProof {
    pub key_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationEnvelope {
    pub holder: Did,
    pub verifiable_credentials: Vec<CredentialEnvelope>,
    pub challenge: Nonce,
    pub domain: String,
    pub proof: PresentationProof,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityCredentialClaims {
    pub authority_did: Did,
    pub role: AuthorityRole,
    pub jurisdiction: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityApproval {
    pub signer: Did,
    pub role: AuthorityRole,
    pub sequence: u16,
    pub credential: CredentialEnvelope,
    pub key_id: String,
    pub signature: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CredentialError {
    #[error("AUTH_MISSING_REQUIRED_ROLE")]
    MissingRequiredRole,
    #[error("AUTH_INVALID_SIGNATURE")]
    InvalidSignature,
    #[error("AUTH_REPLAY_NONCE")]
    ReplayNonce,
    #[error("AUTH_DOMAIN_MISMATCH")]
    DomainMismatch,
    #[error("AUTH_REVOKED_CREDENTIAL")]
    RevokedCredential,
    #[error("AUTH_EXPIRED_CREDENTIAL")]
    ExpiredCredential,
    #[error("AUTH_INVALID_KEY_ID")]
    InvalidKeyId,
    #[error("AUTH_UNEXPECTED_ROLE")]
    UnexpectedRole,
    #[error("AUTH_DUPLICATE_ROLE")]
    DuplicateRole,
    #[error("AUTH_SEQUENCE_INVALID")]
    InvalidSequence,
    #[error("AUTH_CREDENTIAL_INVALID")]
    InvalidCredential,
}

impl From<IdentityError> for CredentialError {
    fn from(value: IdentityError) -> Self {
        match value {
            IdentityError::KeyIdNotFound => Self::InvalidKeyId,
            IdentityError::KeyRevoked | IdentityError::AuthorityNotActive => {
                Self::RevokedCredential
            }
            _ => Self::InvalidCredential,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NonceRegistry(BTreeSet<(String, Nonce)>);

impl NonceRegistry {
    pub fn contains(&self, domain: &str, nonce: &Nonce) -> bool {
        self.0.contains(&(domain.to_owned(), nonce.clone()))
    }

    pub fn record(&mut self, domain: &str, nonce: Nonce) -> Result<(), CredentialError> {
        if !self.0.insert((domain.to_owned(), nonce)) {
            return Err(CredentialError::ReplayNonce);
        }
        Ok(())
    }
}

fn unsigned_credential_bytes(credential: &CredentialEnvelope) -> Result<Vec<u8>, CredentialError> {
    let mut value =
        serde_json::to_value(credential).map_err(|_| CredentialError::InvalidCredential)?;
    value
        .as_object_mut()
        .and_then(|object| object.remove("proof"))
        .ok_or(CredentialError::InvalidCredential)?;
    poa_protocol::canonicalize_value(&value).map_err(|_| CredentialError::InvalidCredential)
}

fn verify_signature(
    store: &LocalIdentityStore,
    signer: &Did,
    key_id: &str,
    at: Timestamp,
    message: &[u8],
    encoded_signature: &str,
) -> Result<(), CredentialError> {
    let key = store.verification_key(signer, key_id, at)?;
    let bytes = STANDARD
        .decode(encoded_signature)
        .map_err(|_| CredentialError::InvalidSignature)?;
    let signature = Signature::from_slice(&bytes).map_err(|_| CredentialError::InvalidSignature)?;
    key.verify(message, &signature)
        .map_err(|_| CredentialError::InvalidSignature)
}

pub fn verify_credential(
    store: &LocalIdentityStore,
    credential: &CredentialEnvelope,
    at: Timestamp,
) -> Result<AuthorityCredentialClaims, CredentialError> {
    if at < credential.issuance_time
        || credential
            .expiration_time
            .is_some_and(|expiry| at >= expiry)
    {
        return Err(CredentialError::ExpiredCredential);
    }
    if credential.status.as_deref() == Some("revoked") {
        return Err(CredentialError::RevokedCredential);
    }
    verify_signature(
        store,
        &credential.issuer,
        &credential.proof.key_id,
        at,
        &unsigned_credential_bytes(credential)?,
        &credential.proof.signature,
    )?;
    serde_json::from_value(credential.claims.clone())
        .map_err(|_| CredentialError::InvalidCredential)
}

/// Verify all authority credentials and signatures before recording the nonce.
/// A failure never consumes the nonce and never returns raw credential material.
pub fn verify_authorization(
    store: &LocalIdentityStore,
    policy: &MachinePolicyObject,
    approvals: &[AuthorityApproval],
    expected_message: &[u8],
    expected_domain: &str,
    now: Timestamp,
    nonces: &mut NonceRegistry,
) -> Result<(), CredentialError> {
    let binding = format!(
        "{}/{}/{}",
        policy.domain_binding.environment_id,
        policy.domain_binding.service_id,
        policy.domain_binding.adapter_id
    );
    if binding != expected_domain {
        return Err(CredentialError::DomainMismatch);
    }
    if nonces.contains(expected_domain, &policy.nonce) {
        return Err(CredentialError::ReplayNonce);
    }

    let expected: BTreeMap<_, _> = policy
        .authorization_policy
        .requirements
        .iter()
        .map(|requirement| (requirement.role.clone(), requirement))
        .collect();
    let mut counts = BTreeMap::<AuthorityRole, u16>::new();
    let mut signer_roles = BTreeMap::<Did, BTreeSet<AuthorityRole>>::new();

    for approval in approvals {
        let requirement = expected
            .get(&approval.role)
            .ok_or(CredentialError::UnexpectedRole)?;
        if policy.authorization_policy.sequential && approval.sequence != requirement.sequence {
            return Err(CredentialError::InvalidSequence);
        }
        let claims = verify_credential(store, &approval.credential, now)?;
        if claims.authority_did != approval.signer || claims.role != approval.role {
            return Err(CredentialError::InvalidCredential);
        }
        if store.authority_status(&approval.signer, &approval.role, now)? != AuthorityStatus::Active
        {
            return Err(CredentialError::RevokedCredential);
        }
        verify_signature(
            store,
            &approval.signer,
            &approval.key_id,
            now,
            expected_message,
            &approval.signature,
        )?;
        *counts.entry(approval.role.clone()).or_default() += 1;
        signer_roles
            .entry(approval.signer.clone())
            .or_default()
            .insert(approval.role.clone());
    }

    enforce_exclusions(&policy.authorization_policy, &signer_roles)?;
    let total: u16 = counts.values().copied().sum();
    if total < policy.authorization_policy.threshold
        || expected.iter().any(|(role, requirement)| {
            counts.get(role).copied().unwrap_or_default() < requirement.minimum_signatures
        })
    {
        return Err(CredentialError::MissingRequiredRole);
    }
    nonces.record(expected_domain, policy.nonce.clone())
}

fn enforce_exclusions(
    policy: &AuthorizationPolicy,
    signer_roles: &BTreeMap<Did, BTreeSet<AuthorityRole>>,
) -> Result<(), CredentialError> {
    for roles in signer_roles.values() {
        for exclusion in &policy.mutually_exclusive_roles {
            if exclusion
                .iter()
                .filter(|role| roles.contains(*role))
                .count()
                > 1
            {
                return Err(CredentialError::DuplicateRole);
            }
        }
    }
    Ok(())
}

pub fn message_digest(message: &[u8]) -> [u8; 32] {
    Sha256::digest(message).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use ed25519_dalek::{Signer, SigningKey};
    use hete_identity::{DidDocument, DidResolver, VerificationMethod};
    use hete_policy::{
        ActionConstraint, AuthorityRef, DomainBinding, EnforcementAction, PolicyDigest, PolicyId,
        PseudonymousTargetRef, ResourceKind, ResourceRef, RevocationRule, RoleRequirement,
        SubjectRef, ValidityWindow,
    };

    fn identity(seed: u8, role: AuthorityRole) -> (SigningKey, Did, LocalIdentityStore) {
        let key = SigningKey::from_bytes(&[seed; 32]);
        let did = Did(format!("did:key:test-{seed}"));
        let mut store = LocalIdentityStore::new(10);
        store.insert_document(DidDocument {
            id: did.clone(),
            verification_methods: vec![VerificationMethod {
                key_id: "key-1".into(),
                public_key: key.verifying_key().to_bytes(),
                activated_at: 0,
                revoked_at: None,
            }],
            updated_at: 0,
        });
        store.grant_role(did.clone(), role, 0, None);
        (key, did, store)
    }

    fn merge_store(
        target: &mut LocalIdentityStore,
        source: LocalIdentityStore,
        did: &Did,
        role: AuthorityRole,
    ) {
        target.insert_document(source.resolve(did).unwrap());
        target.grant_role(did.clone(), role, 0, None);
    }

    fn policy() -> MachinePolicyObject {
        let target = PseudonymousTargetRef::derive("test", "subject", "asset", "w1", b"salt");
        let mut policy = MachinePolicyObject {
            policy_id: PolicyId("p1".into()),
            policy_type: "electronic_warrant".into(),
            version: "1".into(),
            issuer: AuthorityRef {
                did: Did("did:key:issuer".into()),
                role: AuthorityRole::JudicialIssuer,
            },
            authorization_policy: AuthorizationPolicy {
                requirements: vec![
                    RoleRequirement {
                        role: AuthorityRole::Requester,
                        minimum_signatures: 1,
                        sequence: 1,
                    },
                    RoleRequirement {
                        role: AuthorityRole::JudicialIssuer,
                        minimum_signatures: 1,
                        sequence: 2,
                    },
                ],
                threshold: 2,
                mutually_exclusive_roles: vec![vec![
                    AuthorityRole::Requester,
                    AuthorityRole::JudicialIssuer,
                ]],
                sequential: true,
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
                not_before: 0,
                expires_at: 100,
                maximum_duration: 100,
            },
            conditions: vec![],
            obligations: vec![],
            revocation: RevocationRule {
                revocable: true,
                required_role: AuthorityRole::JudicialIssuer,
            },
            evidence_refs: vec![],
            credential_refs: vec![],
            nonce: Nonce("nonce-1".into()),
            domain_binding: DomainBinding {
                environment_id: "env".into(),
                service_id: "svc".into(),
                adapter_id: "adapter".into(),
            },
            policy_digest: PolicyDigest(String::new()),
        };
        policy.seal().unwrap();
        policy
    }

    fn approval(
        key: &SigningKey,
        did: Did,
        role: AuthorityRole,
        sequence: u16,
        message: &[u8],
    ) -> AuthorityApproval {
        let target =
            PseudonymousTargetRef::derive("credential", &did.0, "role", "credential", b"salt");
        let mut credential = CredentialEnvelope {
            id: format!("credential-{}", did.0),
            issuer: did.clone(),
            subject: SubjectRef { target },
            credential_type: vec!["HeteAuthorityCredential".into()],
            issuance_time: 0,
            expiration_time: Some(100),
            status: None,
            claims: serde_json::to_value(AuthorityCredentialClaims {
                authority_did: did.clone(),
                role: role.clone(),
                jurisdiction: "fixture".into(),
            })
            .unwrap(),
            proof: CredentialProof {
                key_id: "key-1".into(),
                signature: String::new(),
            },
        };
        let signature = key.sign(&unsigned_credential_bytes(&credential).unwrap());
        credential.proof.signature = STANDARD.encode(signature.to_bytes());
        AuthorityApproval {
            signer: did,
            role,
            sequence,
            credential,
            key_id: "key-1".into(),
            signature: STANDARD.encode(key.sign(message).to_bytes()),
        }
    }

    fn fixture() -> (
        MachinePolicyObject,
        Vec<AuthorityApproval>,
        LocalIdentityStore,
        Vec<u8>,
    ) {
        let policy = policy();
        let message = b"HETE-EW-V1-bound-message".to_vec();
        let (request_key, request_did, mut store) = identity(1, AuthorityRole::Requester);
        let (judge_key, judge_did, judge_store) = identity(2, AuthorityRole::JudicialIssuer);
        merge_store(
            &mut store,
            judge_store,
            &judge_did,
            AuthorityRole::JudicialIssuer,
        );
        let approvals = vec![
            approval(
                &request_key,
                request_did,
                AuthorityRole::Requester,
                1,
                &message,
            ),
            approval(
                &judge_key,
                judge_did,
                AuthorityRole::JudicialIssuer,
                2,
                &message,
            ),
        ];
        (policy, approvals, store, message)
    }

    #[test]
    fn auth_001_missing_role_rejected() {
        let (policy, mut approvals, store, message) = fixture();
        approvals.pop();
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut NonceRegistry::default()
            ),
            Err(CredentialError::MissingRequiredRole)
        );
    }

    #[test]
    fn auth_002_unexpected_role_rejected() {
        let (policy, mut approvals, mut store, message) = fixture();
        let key = SigningKey::from_bytes(&[1; 32]);
        let did = Did("did:key:test-1".into());
        store.grant_role(did.clone(), AuthorityRole::Auditor, 0, None);
        approvals.push(approval(&key, did, AuthorityRole::Auditor, 3, &message));
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut NonceRegistry::default()
            ),
            Err(CredentialError::UnexpectedRole)
        );
    }

    #[test]
    fn auth_003_same_did_cannot_fill_exclusive_roles() {
        let (policy, mut approvals, mut store, message) = fixture();
        let key = SigningKey::from_bytes(&[1; 32]);
        let did = Did("did:key:test-1".into());
        store.grant_role(did.clone(), AuthorityRole::JudicialIssuer, 0, None);
        approvals[1] = approval(&key, did, AuthorityRole::JudicialIssuer, 2, &message);
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut NonceRegistry::default()
            ),
            Err(CredentialError::DuplicateRole)
        );
    }

    #[test]
    fn auth_004_cross_domain_and_auth_007_replay_rejected() {
        let (policy, approvals, store, message) = fixture();
        let mut nonces = NonceRegistry::default();
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "other/svc/adapter",
                10,
                &mut nonces
            ),
            Err(CredentialError::DomainMismatch)
        );
        assert!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut nonces
            )
            .is_ok()
        );
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut nonces
            ),
            Err(CredentialError::ReplayNonce)
        );
    }

    #[test]
    fn auth_008_changed_message_and_auth_009_key_id_rejected() {
        let (policy, mut approvals, store, message) = fixture();
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                b"changed",
                "env/svc/adapter",
                10,
                &mut NonceRegistry::default()
            ),
            Err(CredentialError::InvalidSignature)
        );
        approvals[0].key_id = "missing".into();
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut NonceRegistry::default()
            ),
            Err(CredentialError::InvalidKeyId)
        );
    }

    #[test]
    fn auth_005_expired_credential_rejected() {
        let (policy, mut approvals, store, message) = fixture();
        approvals[0].credential.expiration_time = Some(10);
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut NonceRegistry::default()
            ),
            Err(CredentialError::ExpiredCredential)
        );
    }

    #[test]
    fn auth_006_revoked_credential_rejected() {
        let (policy, mut approvals, store, message) = fixture();
        approvals[0].credential.status = Some("revoked".into());
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut NonceRegistry::default()
            ),
            Err(CredentialError::RevokedCredential)
        );
    }

    #[test]
    fn auth_010_sequence_is_enforced() {
        let (policy, mut approvals, store, message) = fixture();
        approvals[0].sequence = 2;
        assert_eq!(
            verify_authorization(
                &store,
                &policy,
                &approvals,
                &message,
                "env/svc/adapter",
                10,
                &mut NonceRegistry::default()
            ),
            Err(CredentialError::InvalidSequence)
        );
    }
}
