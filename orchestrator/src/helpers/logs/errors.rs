use ic_agent::export::PrincipalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CanisterBuilderError {
    #[error("Failed to construct wallet canister caller")]
    WalletCanisterCaller(#[source] ic_agent::AgentError),

    #[error("Failed to build call sender")]
    CallSenderBuildError(#[source] ic_agent::AgentError),
}

#[derive(Error, Debug)]
pub enum CallSenderFromWalletError {
    #[error("Failed to read principal from id '{0}', and did not find a wallet for that identity")]
    ParsePrincipalFromIdFailed(String, #[source] PrincipalError),
}
