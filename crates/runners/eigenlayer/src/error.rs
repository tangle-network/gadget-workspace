

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EigenlayerError {
    #[error("AVS Registry error: {0}")]
    AvsRegistryError(#[from] eigensdk::client_avsregistry::error::AvsRegistryError),
    // #[error("IO error: {0}")]
    // Io(#[from] std::io::Error),
    // #[error("Keystore error: {0}")]
    // Keystore(#[from] keystore::Error),
    // #[error("Serialization error: {0}")]
    // Serialization(#[from] bincode::Error),
    // #[error("HTTP error: {0}")]
    // Http(#[from] reqwest::Error),
    // #[error("EcdsaStakeRegistry error: {0}")]
    // EcdsaStakeRegistry(#[from] ecdsa_stake_registry::Error),
    // #[error("IncredibleSquaringTaskManager error: {0}")]
    // IncredibleSquaringTaskManager(#[from] incredible_squaring_task_manager::Error),
    // #[error("RegistryCoordinator error: {0}")]
    // RegistryCoordinator(#[from] registry_coordinator::Error),
}