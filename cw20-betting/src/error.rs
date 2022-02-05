use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unallowed time to bet")]
    UnallowedTime {},

    #[error("Not enough funds set to be able to bet.")]
    NoFunds {},


    #[error("Burn Block is not yet expired")]
    BurnBlock {},

    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
