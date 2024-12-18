use crate::error::SymbioticError;
use alloy_network::EthereumWallet;
use gadget_config::{GadgetConfiguration, ProtocolSettings};
use gadget_runner_core::config::BlueprintConfig;
use gadget_runner_core::error::RunnerError as Error;
use gadget_utils::gadget_utils_evm::{get_provider_http, get_wallet_provider_http};
use symbiotic_rs::OperatorRegistry;

#[derive(Clone, Copy, Default)]
pub struct SymbioticConfig {}

#[async_trait::async_trait]
impl BlueprintConfig for SymbioticConfig {
    async fn requires_registration(&self, env: &GadgetConfiguration) -> Result<bool, Error> {
        let contract_addresses = match env.protocol_settings {
            ProtocolSettings::Symbiotic(addresses) => addresses,
            _ => {
                return Err(gadget_runner_core::error::RunnerError::InvalidProtocol(
                    "Expected Symbiotic protocol".into(),
                ));
            }
        };
        let operator_registry_address = contract_addresses.operator_registry_address;

        let operator_address = env.keystore()?.ecdsa_key()?.alloy_key()?.address();
        let operator_registry = OperatorRegistry::new(
            operator_registry_address,
            get_provider_http(&env.http_rpc_endpoint),
        );

        let is_registered = operator_registry
            .isEntity(operator_address)
            .call()
            .await
            .map(|r| r._0)
            .map_err(|e| SymbioticError::Registration(e.to_string()).into())?;

        Ok(!is_registered)
    }

    async fn register(&self, env: &GadgetConfiguration) -> Result<(), Error> {
        let contract_addresses = match env.protocol_settings {
            ProtocolSettings::Symbiotic(addresses) => addresses,
            _ => {
                return Err(gadget_runner_core::error::RunnerError::InvalidProtocol(
                    "Expected Symbiotic protocol".into(),
                ));
            }
        };
        let operator_registry_address = contract_addresses.operator_registry_address;

        let operator_signer = env.keystore()?.ecdsa_key()?.alloy_key()?;
        let wallet = EthereumWallet::new(operator_signer);
        let provider = get_wallet_provider_http(&env.http_rpc_endpoint, wallet);
        let operator_registry = OperatorRegistry::new(operator_registry_address, provider.clone());

        let result = operator_registry
            .registerOperator()
            .send()
            .await
            .map_err(|e| SymbioticError::Registration(e.to_string()).into())?
            .get_receipt()
            .await
            .map_err(|e| SymbioticError::Registration(e.to_string()).into())?;

        if result.status() {
            gadget_logging::info!("Operator registered successfully");
        } else {
            gadget_logging::error!("Operator registration failed");
        }

        Ok(())
    }
}
