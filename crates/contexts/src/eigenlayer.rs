/// Provides access to Eigenlayer utilities through its [`EigenlayerClient`].
pub trait EigenlayerContext {
    fn client(&self) -> gadget_clients::eigenlayer::client::EigenlayerClient;
}
