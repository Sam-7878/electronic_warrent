use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: only admin can perform this action")]
    Unauthorized {},

    #[error("Judicial signature verification failed for merkle root: {merkle_root}")]
    JudicialSignatureInvalid { merkle_root: String },

    #[error("No judicial public keys registered")]
    NoJudicialKeysRegistered {},

    #[error("Warrant VP data is malformed or incomplete")]
    MalformedWarrantVP {},
}
