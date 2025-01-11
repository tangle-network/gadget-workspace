use crate::contexts::client::AggregatorClient;
use crates::{config::StdGadgetConfiguration, contexts::KeystoreContext};

#[derive(Clone, KeystoreContext)]
pub struct EigenSquareContext {
    pub client: AggregatorClient,
    #[config]
    pub std_config: StdGadgetConfiguration,
}
