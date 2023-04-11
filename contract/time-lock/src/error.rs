use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("NotTokenOfInterest")]
    WrongToken {},

    #[error("Token Received Exceed Contract Specification")]
    TooManyToken {},

    #[error("Reward schedule not start")]
    TooEarlyPleaseChill {},

    #[error("Reward Not Exceed Minimum")]
    TooQuickPleaseChill {},
}
