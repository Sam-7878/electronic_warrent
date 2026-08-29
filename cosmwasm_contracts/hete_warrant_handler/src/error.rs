use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: only admin can perform this action")]
    Unauthorized {},

    #[error("Unauthorized gateway: sender is not a registered judicial gateway")]
    UnauthorizedGateway {},

    #[error("Warrant already submitted for merkle root: {merkle_root}")]
    DuplicateWarrant { merkle_root: String },

    #[error("Warrant not found for merkle root: {merkle_root}")]
    WarrantNotFound { merkle_root: String },

    #[error("Warrant already finalized for merkle root: {merkle_root}")]
    WarrantAlreadyFinalized { merkle_root: String },

    #[error("Unknown reply ID: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Failed to parse reply data from judicial verifier")]
    ReplyDataParseError {},
}
