use cosmwasm_std::StdError;
use thiserror::Error;

/// 투표 시스템 공통 에러 타입
#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: sender is not authorized to perform this action")]
    Unauthorized {},

    #[error("Invalid status transition: cannot transition from current state")]
    InvalidStatusTransition {},

    #[error("Nullifier not found: {nullifier}")]
    NullifierNotFound { nullifier: String },

    #[error("Nullifier already exists: {nullifier}")]
    NullifierAlreadyExists { nullifier: String },

    #[error("Vote session not found: {session_id}")]
    VoteSessionNotFound { session_id: String },

    #[error("Vote session not active: {session_id}")]
    VoteSessionNotActive { session_id: String },

    #[error("Vote session expired: {session_id}")]
    VoteSessionExpired { session_id: String },

    #[error("Duplicate vote: voter has already voted in this session")]
    DuplicateVote {},

    #[error("Invalid candidate index: {index}")]
    InvalidCandidateIndex { index: u32 },

    #[error("TEE attestation verification failed")]
    TeeAttestationFailed {},

    #[error("AnonCreds proof verification failed")]
    AnonCredsProofFailed {},

    #[error("Hardware policy violation: {reason}")]
    HardwarePolicyViolation { reason: String },

    #[error("Reward already processed for nullifier: {nullifier}")]
    RewardAlreadyProcessed { nullifier: String },

    #[error("Reward not found for nullifier: {nullifier}")]
    RewardNotFound { nullifier: String },

    #[error("Unknown reply id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Write-off reconciliation failed: {reason}")]
    ReconciliationFailed { reason: String },

    #[error("VDS authority check failed: unauthorized sender")]
    VdsAuthorityCheckFailed {},
}
