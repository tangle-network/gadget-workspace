pub mod eigenlayer;
pub mod error;

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, U256};
    use eigenlayer::EigenlayerClient;
    use gadget_anvil_utils::{start_anvil_container, wait_transaction, ANVIL_STATE_PATH};
    use gadget_config::GadgetConfiguration;
    use gadget_utils_evm::get_provider_http;

    async fn setup_test_environment() -> (String, String) {
        let (_container, http_endpoint, ws_endpoint) =
            start_anvil_container(ANVIL_STATE_PATH, false).await;
        (http_endpoint, ws_endpoint)
    }

    async fn get_eigenlayer_client() -> EigenlayerClient {
        let (http_endpoint, ws_endpoint) = setup_test_environment().await;
        let config = GadgetConfiguration::default();
    }

    #[tokio::test]
    async fn test_eigenlayer_registry() {
        let (http_endpoint, _) = setup_test_environment().await;
        let provider = get_provider_http(&http_endpoint);
        let client = eigenlayer::EigenLayerClient::new(&http_endpoint)
            .await
            .unwrap();

        // Test getting operator status
        let operator = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let is_operator = client.is_operator(operator).await.unwrap();
        assert!(!is_operator, "Should not be an operator initially");

        // Test getting delegated shares
        let staker = address!("70997970C51812dc3A010C7d01b50e0d17dc79C8");
        let strategy = address!("9965507D1a55bcC2695C58ba16FB37d819B0A4dc");
        let shares = client.get_delegated_shares(staker, strategy).await.unwrap();
        assert_eq!(
            shares,
            U256::ZERO,
            "Should have no delegated shares initially"
        );
    }

    #[tokio::test]
    async fn test_eigenlayer_strategies() {
        let (http_endpoint, _) = setup_test_environment().await;
        let client = eigenlayer::EigenLayerClient::new(&http_endpoint)
            .await
            .unwrap();

        // Test getting strategy details
        let strategy = address!("9965507D1a55bcC2695C58ba16FB37d819B0A4dc");
        let shares = client.get_strategy_shares(strategy).await.unwrap();
        assert_eq!(
            shares,
            U256::ZERO,
            "Should have no strategy shares initially"
        );

        // Test getting underlying token
        let token = client.get_strategy_token(strategy).await.unwrap();
        assert_ne!(
            token,
            address!("0x0000000000000000000000000000000000000000"),
            "Strategy should have an underlying token"
        );
    }

    #[tokio::test]
    async fn test_eigenlayer_delegation() {
        let (http_endpoint, _) = setup_test_environment().await;
        let client = eigenlayer::EigenLayerClient::new(&http_endpoint)
            .await
            .unwrap();

        // Test delegation status
        let delegator = address!("70997970C51812dc3A010C7d01b50e0d17dc79C8");
        let operator = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

        let is_delegated = client.is_delegated(delegator, operator).await.unwrap();
        assert!(!is_delegated, "Should not be delegated initially");

        // Test getting operator details
        let operator_details = client.get_operator_details(operator).await.unwrap();
        assert_eq!(
            operator_details.total_delegated,
            U256::ZERO,
            "Operator should have no delegations initially"
        );
    }

    #[tokio::test]
    async fn test_eigenlayer_staking() {
        let (http_endpoint, _) = setup_test_environment().await;
        let client = eigenlayer::EigenLayerClient::new(&http_endpoint)
            .await
            .unwrap();

        // Test staking status
        let staker = address!("70997970C51812dc3A010C7d01b50e0d17dc79C8");
        let strategy = address!("9965507D1a55bcC2695C58ba16FB37d819B0A4dc");

        let staked_amount = client.get_staked_amount(staker, strategy).await.unwrap();
        assert_eq!(
            staked_amount,
            U256::ZERO,
            "Should have no staked amount initially"
        );

        // Test total staked in strategy
        let total_staked = client.get_total_staked(strategy).await.unwrap();
        assert_eq!(
            total_staked,
            U256::ZERO,
            "Strategy should have no total staked initially"
        );
    }

    #[tokio::test]
    async fn test_eigenlayer_rewards() {
        let (http_endpoint, _) = setup_test_environment().await;
        let client = eigenlayer::EigenLayerClient::new(&http_endpoint)
            .await
            .unwrap();

        // Test reward distribution
        let operator = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let strategy = address!("9965507D1a55bcC2695C58ba16FB37d819B0A4dc");

        let pending_rewards = client
            .get_pending_rewards(operator, strategy)
            .await
            .unwrap();
        assert_eq!(
            pending_rewards,
            U256::ZERO,
            "Should have no pending rewards initially"
        );
    }

    #[tokio::test]
    async fn test_eigenlayer_slashing() {
        let (http_endpoint, _) = setup_test_environment().await;
        let client = eigenlayer::EigenLayerClient::new(&http_endpoint)
            .await
            .unwrap();

        // Test slashing status
        let operator = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

        let frozen = client.is_operator_frozen(operator).await.unwrap();
        assert!(!frozen, "Operator should not be frozen initially");

        // Test slashing history
        let slashing_history = client.get_slashing_history(operator).await.unwrap();
        assert!(
            slashing_history.is_empty(),
            "Should have no slashing history initially"
        );
    }

    #[tokio::test]
    async fn test_eigenlayer_middleware() {
        let (http_endpoint, _) = setup_test_environment().await;
        let client = eigenlayer::EigenLayerClient::new(&http_endpoint)
            .await
            .unwrap();

        // Test middleware registration
        let middleware = address!("9965507D1a55bcC2695C58ba16FB37d819B0A4dc");

        let is_registered = client.is_middleware_registered(middleware).await.unwrap();
        assert!(
            !is_registered,
            "Middleware should not be registered initially"
        );

        // Test middleware operators
        let operators = client.get_middleware_operators(middleware).await.unwrap();
        assert!(operators.is_empty(), "Should have no operators initially");
    }

    #[tokio::test]
    async fn test_eigenlayer_quorum() {
        let (http_endpoint, _) = setup_test_environment().await;
        let client = eigenlayer::EigenLayerClient::new(&http_endpoint)
            .await
            .unwrap();

        // Test quorum status
        let quorum_number = 1u32;

        let quorum_stake = client.get_quorum_stake(quorum_number).await.unwrap();
        assert_eq!(
            quorum_stake,
            U256::ZERO,
            "Quorum should have no stake initially"
        );

        // Test operator quorum membership
        let operator = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
        let is_member = client
            .is_quorum_member(operator, quorum_number)
            .await
            .unwrap();
        assert!(!is_member, "Operator should not be quorum member initially");
    }
}
