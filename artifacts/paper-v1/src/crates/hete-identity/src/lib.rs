//! Deterministic DID resolution and authority trust registry interfaces.
//!
//! Network resolution is deliberately absent from the reference path. A caller
//! must explicitly populate a local resolver and registry, so unavailable or
//! stale external identity data fails closed.

use std::collections::BTreeMap;

use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Timestamp = u64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Did(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityRole {
    Requester,
    LegalReviewer,
    JudicialIssuer,
    Executor,
    Auditor,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationMethod {
    pub key_id: String,
    pub public_key: [u8; 32],
    pub activated_at: Timestamp,
    pub revoked_at: Option<Timestamp>,
}

impl VerificationMethod {
    pub fn key_at(&self, at: Timestamp) -> Result<VerifyingKey, IdentityError> {
        if at < self.activated_at {
            return Err(IdentityError::KeyInactive);
        }
        if self.revoked_at.is_some_and(|revoked| at >= revoked) {
            return Err(IdentityError::KeyRevoked);
        }
        VerifyingKey::from_bytes(&self.public_key).map_err(|_| IdentityError::InvalidKey)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DidDocument {
    pub id: Did,
    pub verification_methods: Vec<VerificationMethod>,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStatus {
    Active,
    Suspended,
    Revoked,
    Unknown,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IdentityError {
    #[error("DID_NOT_FOUND")]
    DidNotFound,
    #[error("KEY_ID_NOT_FOUND")]
    KeyIdNotFound,
    #[error("KEY_INACTIVE")]
    KeyInactive,
    #[error("KEY_REVOKED")]
    KeyRevoked,
    #[error("INVALID_KEY")]
    InvalidKey,
    #[error("STALE_DID_DOCUMENT")]
    StaleDocument,
    #[error("AUTHORITY_NOT_ACTIVE")]
    AuthorityNotActive,
}

/// Resolve a DID document without silently falling back to network state.
pub trait DidResolver {
    fn resolve(&self, did: &Did) -> Result<DidDocument, IdentityError>;
}

/// Determine whether a DID may fulfill a role at the supplied deterministic time.
pub trait TrustRegistry {
    fn authority_status(
        &self,
        did: &Did,
        role: &AuthorityRole,
        at: Timestamp,
    ) -> Result<AuthorityStatus, IdentityError>;
}

#[derive(Debug, Default, Clone)]
pub struct LocalIdentityStore {
    documents: BTreeMap<Did, DidDocument>,
    roles: BTreeMap<(Did, AuthorityRole), Vec<RoleGrant>>,
    pub maximum_document_age: Option<u64>,
    pub current_time: Timestamp,
}

#[derive(Debug, Clone)]
struct RoleGrant {
    active_from: Timestamp,
    revoked_at: Option<Timestamp>,
    suspended: bool,
}

impl LocalIdentityStore {
    pub fn new(current_time: Timestamp) -> Self {
        Self {
            current_time,
            ..Self::default()
        }
    }

    pub fn insert_document(&mut self, document: DidDocument) {
        self.documents.insert(document.id.clone(), document);
    }

    pub fn grant_role(
        &mut self,
        did: Did,
        role: AuthorityRole,
        active_from: Timestamp,
        revoked_at: Option<Timestamp>,
    ) {
        self.roles.entry((did, role)).or_default().push(RoleGrant {
            active_from,
            revoked_at,
            suspended: false,
        });
    }

    pub fn verification_key(
        &self,
        did: &Did,
        key_id: &str,
        at: Timestamp,
    ) -> Result<VerifyingKey, IdentityError> {
        let document = self.resolve(did)?;
        let method = document
            .verification_methods
            .iter()
            .find(|method| method.key_id == key_id)
            .ok_or(IdentityError::KeyIdNotFound)?;
        method.key_at(at)
    }
}

impl DidResolver for LocalIdentityStore {
    fn resolve(&self, did: &Did) -> Result<DidDocument, IdentityError> {
        let document = self
            .documents
            .get(did)
            .cloned()
            .ok_or(IdentityError::DidNotFound)?;
        if self
            .maximum_document_age
            .is_some_and(|age| self.current_time.saturating_sub(document.updated_at) > age)
        {
            return Err(IdentityError::StaleDocument);
        }
        Ok(document)
    }
}

impl TrustRegistry for LocalIdentityStore {
    fn authority_status(
        &self,
        did: &Did,
        role: &AuthorityRole,
        at: Timestamp,
    ) -> Result<AuthorityStatus, IdentityError> {
        let Some(grants) = self.roles.get(&(did.clone(), role.clone())) else {
            return Ok(AuthorityStatus::Unknown);
        };
        let status = grants.iter().find_map(|grant| {
            if at < grant.active_from {
                None
            } else if grant.revoked_at.is_some_and(|revoked| at >= revoked) {
                Some(AuthorityStatus::Revoked)
            } else if grant.suspended {
                Some(AuthorityStatus::Suspended)
            } else {
                Some(AuthorityStatus::Active)
            }
        });
        Ok(status.unwrap_or(AuthorityStatus::Unknown))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoked_key_fails_closed() {
        let method = VerificationMethod {
            key_id: "key-1".into(),
            public_key: [0; 32],
            activated_at: 10,
            revoked_at: Some(20),
        };
        assert_eq!(method.key_at(20), Err(IdentityError::KeyRevoked));
    }
}
