use crate::env::{setup_eigenlayer_test_environment, EigenlayerTestEnvironment};
use crate::runner::EigenlayerBLSTestEnv;
use crate::Error;
use alloy_primitives::Address;
use alloy_provider::RootProvider;
use alloy_transport::BoxTransport;
use gadget_anvil_testing_utils::keys::{inject_anvil_key, ANVIL_PRIVATE_KEYS};
use gadget_anvil_testing_utils::{start_default_anvil_testnet, Container};
use gadget_config::{
    protocol::EigenlayerContractAddresses, supported_chains::SupportedChains, ContextConfig,
    GadgetConfiguration,
};
use gadget_core_testing_utils::harness::{BaseTestHarness, TestHarness};
use gadget_core_testing_utils::runner::TestEnv;
use gadget_macros::ext::event_listeners::core::InitializableEventHandler;
use gadget_runners::core::jobs::JobBuilder;
use gadget_runners::core::runner::BackgroundService;
use gadget_runners::eigenlayer::bls::EigenlayerBLSConfig;
use gadget_utils::evm::get_provider_http;
use url::Url;

/// Configuration for the Eigenlayer test harness
#[derive(Default)]
pub struct EigenlayerTestConfig {
    pub http_endpoint: Option<Url>,
    pub ws_endpoint: Option<Url>,
    pub eigenlayer_contract_addresses: Option<EigenlayerContractAddresses>,
    pub pauser_registry_address: Option<Address>,
}

/// Test harness for Eigenlayer network tests
pub struct EigenlayerTestHarness {
    base: BaseTestHarness<EigenlayerTestConfig>,
    pub http_endpoint: Url,
    pub ws_endpoint: Url,
    pub accounts: Vec<Address>,
    pub eigenlayer_contract_addresses: EigenlayerContractAddresses,
    pub pauser_registry_address: Address,
    _container: Container,
}

#[async_trait::async_trait]
impl TestHarness for EigenlayerTestHarness {
    type Config = EigenlayerTestConfig;
    type Error = Error;

    async fn setup() -> Result<Self, Self::Error> {
        // Start local Anvil testnet
        let (container, http_endpoint, ws_endpoint) = start_default_anvil_testnet(true).await;

        // Setup Eigenlayer test environment
        let EigenlayerTestEnvironment {
            accounts,
            http_endpoint,
            ws_endpoint,
            eigenlayer_contract_addresses,
            pauser_registry_address,
        } = setup_eigenlayer_test_environment(&http_endpoint, &ws_endpoint).await;

        // Setup temporary testing keystore
        let temp_dir = tempfile::TempDir::new()?;
        let temp_dir_path = temp_dir.path().to_string_lossy().into_owned();
        inject_anvil_key(&temp_dir_path, ANVIL_PRIVATE_KEYS[0]).unwrap();

        // Create context config
        let context_config = ContextConfig::create_eigenlayer_config(
            Url::parse(&http_endpoint)?,
            Url::parse(&ws_endpoint)?,
            temp_dir_path,
            None,
            SupportedChains::LocalTestnet,
            eigenlayer_contract_addresses,
        );

        // Load environment
        let env = gadget_macros::ext::config::load(context_config)
            .map_err(|e| Error::Setup(e.to_string()))?;

        // Create config
        let config = EigenlayerTestConfig {
            http_endpoint: Some(Url::parse(&http_endpoint)?),
            ws_endpoint: Some(Url::parse(&ws_endpoint)?),
            eigenlayer_contract_addresses: Some(eigenlayer_contract_addresses),
            pauser_registry_address: Some(pauser_registry_address),
        };

        let base = BaseTestHarness::new(env, config);

        Ok(Self {
            base,
            http_endpoint: Url::parse(&http_endpoint)?,
            ws_endpoint: Url::parse(&ws_endpoint)?,
            accounts,
            eigenlayer_contract_addresses,
            pauser_registry_address,
            _container: container,
        })
    }

    fn env(&self) -> &GadgetConfiguration {
        &self.base.env
    }

    fn config(&self) -> &Self::Config {
        &self.base.config
    }
}

impl EigenlayerTestHarness {
    /// Gets a provider for the HTTP endpoint
    pub fn provider(&self) -> RootProvider<BoxTransport> {
        get_provider_http(self.http_endpoint.as_str())
    }

    /// Gets the list of accounts
    pub fn accounts(&self) -> &[Address] {
        &self.accounts
    }

    /// Gets the owner account (first account)
    pub fn owner_account(&self) -> Address {
        self.accounts[1]
    }

    /// Gets the aggregator account (ninth account)
    pub fn aggregator_account(&self) -> Address {
        self.accounts[9]
    }

    /// Gets the task generator account (fourth account)
    pub fn task_generator_account(&self) -> Address {
        self.accounts[4]
    }
}
