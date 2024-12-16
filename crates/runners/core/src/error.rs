


#[derive(thiserror::Error, Debug)]
pub enum RunnerError {
    #[error(transparent)]

    #[cfg(feature = "eigenlayer")]
    #[error(transparent)]
    EigenlayerError(#[from] gadget_eigenlayer::error::Error),
    #[cfg(feature = "symbiotic")]
    #[error(transparent)]
    SymbioticError(#[from] gadget_symbiotic::error::Error),
    #[cfg(feature = "tangle")]
    #[error(transparent)]
    TangleError(#[from] gadget_tangle::error::Error),
}

// #[derive(thiserror::Error, Debug)]
// pub enum RunnerError {
//     #[error("No jobs registered. Make sure to add a job with `BlueprintRunner::add_job`")]
//     NoJobs,
//     #[error("Job already initialized")]
//     AlreadyInitialized,
//
//     #[error("You are currently not an active operator\nPlease checkout the docs here: https://docs.tangle.tools/restake/join_operator/join")]
//     NotActiveOperator,
//
//     #[error(transparent)]
//     Recv(#[from] tokio::sync::oneshot::error::RecvError),
//
//     #[error("Environment not set")]
//     EnvNotSet,
//
//     #[error("Receiver error")]
//     ReceiverError,
//
//     #[error(transparent)]
//     ConfigError(#[from] crate::config::Error),
//
//     #[error(transparent)]
//     SubxtError(#[from] subxt::Error),
//
//     #[error(transparent)]
//     KeystoreError(#[from] crate::keystore::Error),
//
//     #[error(transparent)]
//     ContractError(#[from] alloy_contract::Error),
//
//     #[error(transparent)]
//     PendingTransactionError(#[from] alloy_provider::PendingTransactionError),
//
//     #[error(transparent)]
//     ElContractsError(#[from] eigensdk::client_elcontracts::error::ElContractsError),
//
//     #[error(transparent)]
//     AvsRegistryError(#[from] eigensdk::client_avsregistry::error::AvsRegistryError),
//
//     #[error("Transaction error: {0}")]
//     TransactionError(String),
//
//     #[error(transparent)]
//     TransportError(#[from] alloy_transport::RpcError<alloy_transport::TransportErrorKind>),
//
//     #[error("Environment not set")]
//     EnvironmentNotSet,
//
//     #[error("Eigenlayer error: {0}")]
//     EigenlayerError(String),
//
//     #[error("Signature error: {0}")]
//     SignatureError(String),
//
//     #[error("Symbiotic error: {0}")]
//     SymbioticError(String),
//
//     #[error("Invalid protocol: {0}")]
//     InvalidProtocol(String),
//
//     #[error("Storage error: {0}")]
//     StorageError(String),
// }