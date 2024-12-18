use crate::error::TangleError;
use gadget_clients::tangle::runtime::TangleClient;
use gadget_config::{GadgetConfiguration, ProtocolSettings};
use gadget_keystore::backends::tangle::{TangleBackend, TanglePairSigner};
use gadget_keystore::{Keystore, KeystoreConfig};
use gadget_runner_core::config::BlueprintConfig;
use gadget_runner_core::error::{RunnerError as Error, RunnerError};
use gadget_std::string::ToString;
use sp_core::Pair;
use subxt::ext::futures::future::select_ok;
use subxt::PolkadotConfig;
use tangle_subxt::tangle_testnet_runtime::api;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_primitives::services;
use tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_primitives::services::PriceTargets as TanglePriceTargets;
use tangle_subxt::tangle_testnet_runtime::api::services::calls::types::register::RegistrationArgs;

/// Wrapper for `tangle_subxt`'s [`PriceTargets`]
///
/// This provides a [`Default`] impl for a zeroed-out [`PriceTargets`].
///
/// [`PriceTargets`]: tangle_subxt::tangle_testnet_runtime::api::runtime_types::tangle_primitives::services::PriceTargets
#[derive(Clone)]
pub struct PriceTargets(TanglePriceTargets);

impl From<TanglePriceTargets> for PriceTargets {
    fn from(t: TanglePriceTargets) -> Self {
        PriceTargets(t)
    }
}

impl Default for PriceTargets {
    fn default() -> Self {
        Self(TanglePriceTargets {
            cpu: 0,
            mem: 0,
            storage_hdd: 0,
            storage_ssd: 0,
            storage_nvme: 0,
        })
    }
}

#[derive(Clone, Default)]
pub struct TangleConfig {
    pub price_targets: PriceTargets,
}

#[async_trait::async_trait]
impl BlueprintConfig for TangleConfig {
    async fn requires_registration(&self, env: &GadgetConfiguration) -> Result<bool, Error> {
        requires_registration_impl(env).await
    }

    async fn register(&self, env: &GadgetConfiguration) -> Result<(), Error> {
        register_impl(self.clone().price_targets, vec![], env).await
    }
}

pub async fn requires_registration_impl(env: &GadgetConfiguration) -> Result<bool, Error> {
    let blueprint_id = match env.protocol_settings {
        ProtocolSettings::Tangle(settings) => settings.blueprint_id,
        _ => {
            return Err(RunnerError::InvalidProtocol(
                "Expected Tangle protocol".into(),
            ))
        }
    };

    // Check if the operator is already registered
    let client = get_client(env.ws_rpc_endpoint.as_str(), env.http_rpc_endpoint.as_str()).await?;

    // TODO: Improve key fetching logic
    let keystore_path = env.clone().keystore_uri;
    let keystore_config = KeystoreConfig::new()
        .in_memory(false)
        .fs_root(keystore_path);
    let keystore = Keystore::new(keystore_config).unwrap();

    let sr25519_key = keystore.iter_sr25519().next().unwrap();
    let sr25519_pair = keystore
        .expose_sr25519_secret(&sr25519_key)
        .unwrap()
        .unwrap();
    let signer: subxt::tx::PairSigner<PolkadotConfig, sp_core::sr25519::Pair> =
        subxt::tx::PairSigner::new(sr25519_pair);

    let account_id = signer.account_id();

    let operator_profile_query = api::storage().services().operators_profile(account_id);
    let operator_profile = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| <TangleError as Into<RunnerError>>::into(TangleError::Network(e.to_string())))?
        .fetch(&operator_profile_query)
        .await
        .map_err(|e| {
            <TangleError as Into<RunnerError>>::into(TangleError::Network(e.to_string()))
        })?;
    let is_registered = operator_profile
        .map(|p| p.blueprints.0.iter().any(|&id| id == blueprint_id))
        .unwrap_or(false);

    Ok(!is_registered)
}

pub async fn register_impl(
    price_targets: PriceTargets,
    registration_args: RegistrationArgs,
    env: &GadgetConfiguration,
) -> Result<(), RunnerError> {
    let client = get_client(env.ws_rpc_endpoint.as_str(), env.http_rpc_endpoint.as_str()).await?;

    // TODO: Improve key fetching logic
    let keystore_path = env.clone().keystore_uri;
    let keystore_config = KeystoreConfig::new()
        .in_memory(false)
        .fs_root(keystore_path);
    let keystore = Keystore::new(keystore_config).unwrap();

    let sr25519_key = keystore.iter_sr25519().next().unwrap();
    let sr25519_pair = keystore
        .expose_sr25519_secret(&sr25519_key)
        .unwrap()
        .unwrap();
    let signer = subxt::tx::PairSigner::new(sr25519_pair);

    let ecdsa_key = keystore.iter_ecdsa().next().unwrap();
    let ecdsa_pair = keystore.expose_ecdsa_secret(&ecdsa_key).unwrap().unwrap();
    let ecdsa_pair = TanglePairSigner {
        pair: subxt::tx::PairSigner::new(ecdsa_pair),
    };

    // Parse Tangle protocol specific settings
    let ProtocolSettings::Tangle(blueprint_settings) = env.protocol_settings else {
        return Err(RunnerError::InvalidProtocol(
            "Expected Tangle protocol".into(),
        ));
    };

    let account_id = signer.account_id();
    // Check if the operator is active operator.
    let operator_active_query = api::storage()
        .multi_asset_delegation()
        .operators(account_id);
    let operator_active = client
        .storage()
        .at_latest()
        .await
        .map_err(|e| <TangleError as Into<RunnerError>>::into(TangleError::Network(e.to_string())))?
        .fetch(&operator_active_query)
        .await
        .map_err(|e| {
            <TangleError as Into<RunnerError>>::into(TangleError::Network(e.to_string()))
        })?;
    if operator_active.is_none() {
        return Err(RunnerError::NotActiveOperator);
    }

    let blueprint_id = blueprint_settings.blueprint_id;

    let xt = api::tx().services().register(
        blueprint_id,
        services::OperatorPreferences {
            key: ecdsa_pair.pair.signer().public().0,
            price_targets: price_targets.clone().0,
        },
        registration_args,
        0,
    );

    // send the tx to the tangle and exit.
    let result = gadget_utils::gadget_utils_tangle::tx::send(&client, &signer, &xt)
        .await
        .map_err(|e| {
            <TangleError as Into<RunnerError>>::into(TangleError::Network(e.to_string()))
        })?;
    gadget_logging::info!("Registered operator with hash: {:?}", result);
    Ok(())
}

async fn get_client(ws_url: &str, http_url: &str) -> Result<TangleClient, RunnerError> {
    let task0 = TangleClient::from_url(ws_url);
    let task1 = TangleClient::from_url(http_url);
    Ok(select_ok([Box::pin(task0), Box::pin(task1)])
        .await
        .map_err(|e| {
            <crate::error::TangleError as Into<RunnerError>>::into(
                crate::error::TangleError::Network(e.to_string()),
            )
        })?
        .0)
}
