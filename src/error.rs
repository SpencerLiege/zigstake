use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("Round not started")]
    RoundNotStarted {},

    #[error("The contract is currently pasued")]
    ContractPaused {},

    #[error("Round locked")]
    RoundLocked {},

    #[error("User already placed bet")]
    BetAlreadyPlaced {},

    #[error("No fund sent")]
    NoFundSent {},

    #[error("Round not ended")]
    RoundNotEnded {},

    #[error("Bet not found")]
    BetNotFound {},

    #[error("Round not found")]
    RoundNotFound {},

    #[error("Cannot lock round before lock time")]
    CannotLockBeforeTime {},
    
    #[error("Cannnot end round without lock price")]
    CannnotEndWithoutLockPrice {},

    #[error("Cannot end round before end time")]
    CannotEndBeforeTime {},

    #[error("Cannnot start new round")]
    CannotStartNewRound {},
}
