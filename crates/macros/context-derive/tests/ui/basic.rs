use async_trait::async_trait;
use gadget_config::{GadgetConfiguration, StdGadgetConfiguration};
use gadget_context_derive::{
    EVMProviderContext, KeystoreContext, P2pContext, ServicesContext, TangleClientContext,
};
use gadget_contexts::instrumented_evm_client::EvmInstrumentedClientContext as _;
use gadget_contexts::keystore::KeystoreContext as _;
use gadget_contexts::tangle::TangleClientContext as _;
use gadget_networking::networking::{Network, NetworkMultiplexer, ProtocolMessage};
use gadget_stores::local_database::LocalDatabase;
use round_based::ProtocolMessage as RoundBasedProtocolMessage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use subxt_core::ext::sp_core::ecdsa::Public;

#[derive(KeystoreContext, EVMProviderContext, TangleClientContext, ServicesContext, P2pContext)]
#[allow(dead_code)]
struct MyContext {
    foo: String,
    #[config]
    config: StdGadgetConfiguration,
    store: Arc<LocalDatabase<u64>>,
    #[call_id]
    call_id: Option<u64>,
}

#[allow(dead_code)]
fn main() {
    let body = async {
        let ctx = MyContext {
            foo: "bar".to_string(),
            config: GadgetConfiguration::default(),
            store: Arc::new(LocalDatabase::open("test.json")),
            call_id: None,
        };

        // Test existing context functions
        let _keystore = ctx.keystore();
        let _evm_provider = ctx.evm_client();
        let tangle_client = ctx.tangle_client().await.unwrap();
        let _services = ctx.current_service_operators(&tangle_client).await.unwrap();

        // Test MPC context utility functions
        let _config = ctx.config();
        let _protocol = ctx.network_protocol();

        // Test MPC context functions

        let mux = Arc::new(NetworkMultiplexer::new(StubNetwork));
        let party_index = 0;
        let task_hash = [0u8; 32];
        let mut parties = BTreeMap::<u16, _>::new();
        parties.insert(0, Public([0u8; 33]));

        // Test network delivery wrapper creation
        let _network_wrapper = ctx.create_network_delivery_wrapper::<StubMessage>(
            mux.clone(),
            party_index,
            task_hash,
            parties.clone(),
        );

        // Test party index retrieval
        let _party_idx = ctx.get_party_index().await;

        // Test participants retrieval
        let _participants = ctx.get_participants(&tangle_client).await;

        // Test blueprint ID retrieval
        let _blueprint_id = tangle_client.blueprint_id();

        // Test party index and operators retrieval
        let _party_idx_ops = ctx.get_party_index_and_operators().await;

        // Test service operators ECDSA keys retrieval
        let _operator_keys = ctx.current_service_operators_ecdsa_keys().await;

        // Test current call ID retrieval
        let _call_id = tangle_client.current_call_id().await;
    };

    drop(body);
}

#[derive(RoundBasedProtocolMessage, Clone, Serialize, Deserialize)]
enum StubMessage {}

#[allow(dead_code)]
struct StubNetwork;

#[async_trait]
impl Network for StubNetwork {
    async fn next_message(&self) -> Option<ProtocolMessage> {
        None
    }

    async fn send_message(
        &self,
        message: ProtocolMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        drop(message);
        Ok(())
    }
}
