use crate::contexts::client::AggregatorClient;
use gadget_config::StdGadgetConfiguration;
use gadget_macros::contexts::KeystoreContext;

#[derive(Clone, KeystoreContext)]
pub struct EigenSquareContext {
    pub client: AggregatorClient,
    #[config]
    pub std_config: StdGadgetConfiguration,
}
