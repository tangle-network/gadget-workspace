use gadget_std::fmt::Debug;
use gadget_std::string::{String, ToString};

#[cfg(feature = "std")]
use gadget_std::path::PathBuf;
#[cfg(not(feature = "std"))]
pub type PathBuf = String;

pub mod context_config;
pub mod protocol;
pub mod supported_chains;

pub use context_config::{ContextConfig, GadgetCLICoreSettings};
pub use protocol::{Protocol, ProtocolSettings};

// TODO: This is temporary to continue work with runners without the keystore completed
#[cfg(all(feature = "keystore", feature = "std"))]
impl GadgetConfiguration {
    /// Creates a Keystore from the environment.
    /// # Errors
    ///
    /// This function will return an error if any of the required environment variables are missing.
    pub fn keystore(&self) -> Result<gadget_keystore::Keystore, Error> {
        let config = gadget_keystore::KeystoreConfig::new().fs_root(self.clone().keystore_uri);
        gadget_keystore::Keystore::new(config)
            .map_err(|e| Error::UnsupportedKeystoreUri(e.to_string()))
    }
}

/// Errors that can occur while loading and using the gadget configuration.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Missing `RPC_URL` environment variable.
    #[error("Missing Tangle RPC endpoint")]
    MissingTangleRpcEndpoint,
    /// Missing `KEYSTORE_URI` environment
    #[error("Missing keystore URI")]
    MissingKeystoreUri,
    /// Missing `BLUEPRINT_ID` environment variable
    #[error("Missing blueprint ID")]
    MissingBlueprintId,
    /// Missing `SERVICE_ID` environment variable
    #[error("Missing service ID")]
    MissingServiceId,
    /// Error parsing the blueprint ID.
    #[error(transparent)]
    MalformedBlueprintId(core::num::ParseIntError),
    /// Error parsing the service ID.
    #[error(transparent)]
    MalformedServiceId(core::num::ParseIntError),
    /// Unsupported keystore URI.
    #[error("Unsupported keystore URI: {0}")]
    UnsupportedKeystoreUri(String),
    /// Error parsing the protocol, from the `PROTOCOL` environment variable.
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    /// No Sr25519 keypair found in the keystore.
    #[error("No Sr25519 keypair found in the keystore")]
    NoSr25519Keypair,
    /// Invalid Sr25519 keypair found in the keystore.
    #[error("Invalid Sr25519 keypair found in the keystore")]
    InvalidSr25519Keypair,
    /// No ECDSA keypair found in the keystore.
    #[error("No ECDSA keypair found in the keystore")]
    NoEcdsaKeypair,
    /// Invalid ECDSA keypair found in the keystore.
    #[error("Invalid ECDSA keypair found in the keystore")]
    InvalidEcdsaKeypair,
    /// Test setup error
    #[error("Test setup error: {0}")]
    TestSetup(String),
    /// Missing `EigenlayerContractAddresses`
    #[error("Missing EigenlayerContractAddresses")]
    MissingEigenlayerContractAddresses,
    /// Missing `SymbioticContractAddresses`
    #[error("Missing SymbioticContractAddresses")]
    MissingSymbioticContractAddresses,
    #[error("Bad RPC Connection: {0}")]
    BadRpcConnection(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

#[cfg(feature = "networking")]
use libp2p::Multiaddr;

pub type StdGadgetConfiguration = GadgetConfiguration;

/// Gadget environment.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct GadgetConfiguration {
    /// HTTP RPC endpoint for host restaking network (Tangle / Ethereum (Eigenlayer or Symbiotic)).
    pub http_rpc_endpoint: String,
    /// WS RPC endpoint for host restaking network (Tangle / Ethereum (Eigenlayer or Symbiotic)).
    pub ws_rpc_endpoint: String,
    /// The keystore URI for the gadget
    pub keystore_uri: String,
    /// Data directory exclusively for this gadget
    ///
    /// This will be `None` if the blueprint manager was not provided a base directory.
    pub data_dir: Option<PathBuf>,
    /// The list of bootnodes to connect to
    #[cfg(feature = "networking")]
    pub bootnodes: Vec<Multiaddr>,
    /// The type of protocol the gadget is executing on.
    pub protocol: Protocol,
    /// Protocol-specific settings
    pub protocol_settings: ProtocolSettings,
    /// Whether the gadget is in test mode
    pub test_mode: bool,
}

/// Loads the [`GadgetConfiguration`] from the current environment.
/// # Errors
///
/// This function will return an error if any of the required environment variables are missing.
pub fn load(config: ContextConfig) -> Result<GadgetConfiguration, Error> {
    load_inner(config)
}

fn load_inner(config: ContextConfig) -> Result<GadgetConfiguration, Error> {
    tracing::info_span!("gadget");
    let ContextConfig {
        gadget_core_settings:
            GadgetCLICoreSettings::Run {
                test_mode,
                http_rpc_url,
                ws_rpc_url,
                #[cfg(feature = "networking")]
                bootnodes,
                keystore_uri,
                protocol,
                #[cfg(feature = "tangle")]
                blueprint_id,
                #[cfg(feature = "tangle")]
                service_id,
                #[cfg(feature = "eigenlayer")]
                registry_coordinator,
                #[cfg(feature = "eigenlayer")]
                operator_state_retriever,
                #[cfg(feature = "eigenlayer")]
                delegation_manager,
                #[cfg(feature = "eigenlayer")]
                service_manager,
                #[cfg(feature = "eigenlayer")]
                stake_registry,
                #[cfg(feature = "eigenlayer")]
                strategy_manager,
                #[cfg(feature = "eigenlayer")]
                avs_directory,
                #[cfg(feature = "eigenlayer")]
                rewards_coordinator,
                #[cfg(feature = "symbiotic")]
                operator_registry,
                #[cfg(feature = "symbiotic")]
                network_registry,
                #[cfg(feature = "symbiotic")]
                base_delegator,
                #[cfg(feature = "symbiotic")]
                network_opt_in_service,
                #[cfg(feature = "symbiotic")]
                vault_opt_in_service,
                #[cfg(feature = "symbiotic")]
                slasher,
                #[cfg(feature = "symbiotic")]
                veto_slasher,
                ..
            },
        ..
    } = config;

    let protocol_settings = if cfg!(feature = "tangle") && matches!(protocol, Protocol::Tangle) {
        #[cfg(feature = "tangle")]
        {
            ProtocolSettings::from_tangle(crate::protocol::TangleInstanceSettings {
                blueprint_id: blueprint_id.ok_or(Error::MissingBlueprintId)?,
                service_id: Some(service_id.ok_or(Error::MissingServiceId)?),
            })
        }
        #[cfg(not(feature = "tangle"))]
        {
            return Err(Error::UnsupportedProtocol("tangle".to_string()));
        }
    } else if cfg!(feature = "eigenlayer") && matches!(protocol, Protocol::Eigenlayer) {
        #[cfg(feature = "eigenlayer")]
        {
            ProtocolSettings::from_eigenlayer(crate::protocol::EigenlayerContractAddresses {
                registry_coordinator_address: registry_coordinator
                    .ok_or(Error::MissingEigenlayerContractAddresses)?,
                operator_state_retriever_address: operator_state_retriever
                    .ok_or(Error::MissingEigenlayerContractAddresses)?,
                delegation_manager_address: delegation_manager
                    .ok_or(Error::MissingEigenlayerContractAddresses)?,
                service_manager_address: service_manager
                    .ok_or(Error::MissingEigenlayerContractAddresses)?,
                stake_registry_address: stake_registry
                    .ok_or(Error::MissingEigenlayerContractAddresses)?,
                strategy_manager_address: strategy_manager
                    .ok_or(Error::MissingEigenlayerContractAddresses)?,
                avs_directory_address: avs_directory
                    .ok_or(Error::MissingEigenlayerContractAddresses)?,
                rewards_coordinator_address: rewards_coordinator
                    .ok_or(Error::MissingEigenlayerContractAddresses)?,
            })
        }
        #[cfg(not(feature = "eigenlayer"))]
        {
            return Err(Error::UnsupportedProtocol("eigenlayer".to_string()));
        }
    } else if cfg!(feature = "symbiotic") && matches!(protocol, Protocol::Symbiotic) {
        #[cfg(feature = "symbiotic")]
        {
            ProtocolSettings::from_symbiotic(crate::protocol::SymbioticContractAddresses {
                operator_registry_address: operator_registry
                    .ok_or(Error::MissingSymbioticContractAddresses)?,
                network_registry_address: network_registry
                    .ok_or(Error::MissingSymbioticContractAddresses)?,
                base_delegator_address: base_delegator
                    .ok_or(Error::MissingSymbioticContractAddresses)?,
                network_opt_in_service_address: network_opt_in_service
                    .ok_or(Error::MissingSymbioticContractAddresses)?,
                vault_opt_in_service_address: vault_opt_in_service
                    .ok_or(Error::MissingSymbioticContractAddresses)?,
                slasher_address: slasher.ok_or(Error::MissingSymbioticContractAddresses)?,
                veto_slasher_address: veto_slasher
                    .ok_or(Error::MissingSymbioticContractAddresses)?,
            })
        }
        #[cfg(not(feature = "symbiotic"))]
        {
            return Err(Error::UnsupportedProtocol("symbiotic".to_string()));
        }
    } else {
        return Err(Error::UnsupportedProtocol(protocol.to_string()));
    };

    Ok(GadgetConfiguration {
        test_mode,
        http_rpc_endpoint: http_rpc_url.to_string(),
        ws_rpc_endpoint: ws_rpc_url.to_string(),
        keystore_uri,
        #[cfg(feature = "std")]
        data_dir: gadget_std::env::var("DATA_DIR").ok().map(PathBuf::from),
        #[cfg(not(feature = "std"))]
        data_dir: None,
        #[cfg(feature = "networking")]
        bootnodes: bootnodes.unwrap_or_default(),
        protocol,
        protocol_settings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        ContextConfig::command().debug_assert();
    }
}
