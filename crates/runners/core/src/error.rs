#[derive(thiserror::Error, Debug)]
pub enum RunnerError {
    #[error(transparent)]
    Recv(#[from] tokio::sync::oneshot::error::RecvError),
    #[error(transparent)]
    InvalidProtocol(#[from] String),
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