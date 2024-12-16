use alloy_primitives::Address;
use gadget_config::GadgetConfiguration;
use gadget_runner_core::config::BlueprintConfig;
use crate::error::EigenlayerError as Error;

#[derive(Clone, Copy)]
pub struct EigenlayerBLSConfig {
    earnings_receiver_address: Address,
    delegation_approver_address: Address,
}

impl EigenlayerBLSConfig {
    pub fn new(earnings_receiver_address: Address, delegation_approver_address: Address) -> Self {
        Self {
            earnings_receiver_address,
            delegation_approver_address,
        }
    }
}

#[async_trait::async_trait]
impl BlueprintConfig for EigenlayerBLSConfig {
    async fn register(
        &self,
        env: &GadgetConfiguration,
    ) -> Result<(), Error> {
        register_bls_impl(
            env,
            self.earnings_receiver_address,
            self.delegation_approver_address,
        )
            .await
    }

    async fn requires_registration(
        &self,
        env: &GadgetConfiguration,
    ) -> Result<bool, Error> {
        requires_registration_bls_impl(env).await
    }
}

async fn requires_registration_bls_impl(
    env: &GadgetConfiguration,
) -> Result<bool, Error> {
    if env.skip_registration {
        return Ok(false);
    }

    let ProtocolSpecificSettings::Eigenlayer(contract_addresses) = &env.protocol_specific else {
        return Err(Error::InvalidProtocol(
            "Expected Eigenlayer protocol".into(),
        ));
    };
    let registry_coordinator_address = contract_addresses.registry_coordinator_address;
    let operator_state_retriever_address = contract_addresses.operator_state_retriever_address;
    let operator = env.keystore()?.ecdsa_key()?;
    let operator_address = operator.alloy_key()?.address();

    let avs_registry_reader = eigensdk::client_avsregistry::reader::AvsRegistryChainReader::new(
        get_test_logger(),
        registry_coordinator_address,
        operator_state_retriever_address,
        env.http_rpc_endpoint.clone(),
    )
        .await?;

    // Check if the operator has already registered for the service
    match avs_registry_reader
        .is_operator_registered(operator_address)
        .await
    {
        Ok(is_registered) => Ok(!is_registered),
        Err(e) => Err(Error::AvsRegistryError(e)),
    }
}

async fn register_bls_impl(
    env: &GadgetConfiguration<parking_lot::RawRwLock>,
    earnings_receiver_address: Address,
    delegation_approver_address: Address,
) -> Result<(), Error> {
    if env.test_mode {
        info!("Skipping registration in test mode");
        return Ok(());
    }

    let ProtocolSpecificSettings::Eigenlayer(contract_addresses) = &env.protocol_specific else {
        return Err(Error::InvalidProtocol(
            "Expected Eigenlayer protocol".into(),
        ));
    };

    let registry_coordinator_address = contract_addresses.registry_coordinator_address;
    let operator_state_retriever_address = contract_addresses.operator_state_retriever_address;
    let delegation_manager_address = contract_addresses.delegation_manager_address;
    let strategy_manager_address = contract_addresses.strategy_manager_address;
    let rewards_coordinator_address = contract_addresses.rewards_coordinator_address;
    let avs_directory_address = contract_addresses.avs_directory_address;

    let operator = env.keystore()?.ecdsa_key()?;
    let operator_private_key = hex::encode(operator.signer().seed());
    let operator_address = operator.alloy_key()?.address();
    let provider = get_provider_http(&env.http_rpc_endpoint);

    let delegation_manager =
        eigensdk::utils::delegationmanager::DelegationManager::DelegationManagerInstance::new(
            delegation_manager_address,
            provider.clone(),
        );
    let slasher_address = delegation_manager.slasher().call().await.map(|a| a._0)?;

    let logger = get_test_logger();
    let avs_registry_writer = AvsRegistryChainWriter::build_avs_registry_chain_writer(
        logger.clone(),
        env.http_rpc_endpoint.clone(),
        operator_private_key.clone(),
        registry_coordinator_address,
        operator_state_retriever_address,
    )
        .await
        .expect("avs writer build fail ");

    let operator_bls_key = env.keystore()?.bls_bn254_key()?;
    let digest_hash: FixedBytes<32> = FixedBytes::from([0x02; 32]);

    let now = std::time::SystemTime::now();
    let sig_expiry = now
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| U256::from(duration.as_secs()) + U256::from(86400))
        .unwrap_or_else(|_| {
            info!("System time seems to be before the UNIX epoch.");
            U256::from(0)
        });

    let quorum_nums = Bytes::from(vec![0]);

    let el_chain_reader = ELChainReader::new(
        logger,
        slasher_address,
        delegation_manager_address,
        avs_directory_address,
        env.http_rpc_endpoint.clone(),
    );

    let el_writer = ELChainWriter::new(
        delegation_manager_address,
        strategy_manager_address,
        rewards_coordinator_address,
        el_chain_reader,
        env.http_rpc_endpoint.clone(),
        operator_private_key,
    );

    let staker_opt_out_window_blocks = 50400u32;
    let operator_details = Operator {
        address: operator_address,
        earnings_receiver_address,
        delegation_approver_address,
        metadata_url: Some("https://github.com/tangle-network/gadget".to_string()),
        staker_opt_out_window_blocks,
    };

    let tx_hash = el_writer.register_as_operator(operator_details).await?;
    info!("Registered as operator for Eigenlayer {:?}", tx_hash);

    let tx_hash = avs_registry_writer
        .register_operator_in_quorum_with_avs_registry_coordinator(
            operator_bls_key,
            digest_hash,
            sig_expiry,
            quorum_nums,
            env.http_rpc_endpoint.clone(),
        )
        .await?;

    info!("Registered operator for Eigenlayer {:?}", tx_hash);
    Ok(())
}
