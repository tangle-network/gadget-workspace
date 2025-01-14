use crate::anvil::start_anvil_container;
use testcontainers::{ContainerAsync, GenericImage};

/// Starts an Anvil container for testing from the given state file in JSON format.
///
/// # Arguments
/// * `path` - The path to the save-state file.
/// * `include_logs` - If true, testnet output will be printed to the console.
///
/// # Returns
/// `(container, http_endpoint, ws_endpoint)`
///    - `container` as a [`ContainerAsync`] - The Anvil container.
///    - `http_endpoint` as a `String` - The Anvil HTTP endpoint.
///    - `ws_endpoint` as a `String` - The Anvil WS endpoint.
pub async fn start_anvil_testnet(
    path: &str,
    include_logs: bool,
) -> (ContainerAsync<GenericImage>, String, String) {
    let (container, http_endpoint, ws_endpoint) =
        anvil::start_anvil_container(path, include_logs).await;
    std::env::set_var("EIGENLAYER_HTTP_ENDPOINT", http_endpoint.clone());
    std::env::set_var("EIGENLAYER_WS_ENDPOINT", ws_endpoint.clone());

    // Sleep to give the testnet time to spin up
    tokio::time::sleep(Duration::from_secs(1)).await;
    (container, http_endpoint, ws_endpoint)
}

/// Starts an Anvil container for testing from this library's default state file.
///
/// # Arguments
/// * `include_logs` - If true, testnet output will be printed to the console.
///
/// # Returns
/// `(container, http_endpoint, ws_endpoint)`
///    - `container` as a [`ContainerAsync`] - The Anvil container.
///    - `http_endpoint` as a `String` - The Anvil HTTP endpoint.
///    - `ws_endpoint` as a `String` - The Anvil WS endpoint.
pub async fn start_default_anvil_testnet(
    include_logs: bool,
) -> (ContainerAsync<GenericImage>, String, String) {
    info!("Starting Anvil testnet from default state file");
    anvil::start_anvil_container(DEFAULT_ANVIL_STATE_PATH, include_logs).await
}

pub async fn get_receipt<T, P, D>(
    call: CallBuilder<T, P, D, Ethereum>,
) -> Result<TransactionReceipt, BlueprintError>
where
    T: Transport + Clone,
    P: Provider<T, Ethereum>,
    D: CallDecoder,
{
    let pending_tx = match call.send().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to send transaction: {:?}", e);
            return Err(e.into());
        }
    };

    let receipt = match pending_tx.get_receipt().await {
        Ok(receipt) => receipt,
        Err(e) => {
            error!("Failed to get transaction receipt: {:?}", e);
            return Err(e.into());
        }
    };

    Ok(receipt)
}

/// Waits for the given `successful_responses` Mutex to be greater than or equal to `task_response_count`.
pub async fn wait_for_responses(
    successful_responses: Arc<Mutex<usize>>,
    task_response_count: usize,
    timeout_duration: Duration,
) -> Result<Result<(), Error>, tokio::time::error::Elapsed> {
    tokio::time::timeout(timeout_duration, async move {
        loop {
            let count = *successful_responses.lock().await;
            if count >= task_response_count {
                crate::info!("Successfully received {} task responses", count);
                return Ok(());
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
        .await
}