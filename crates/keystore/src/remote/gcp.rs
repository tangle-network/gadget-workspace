use super::{EcdsaRemoteSigner, RemoteConfig};
use crate::error::{Error, Result};
use alloy_primitives::keccak256;
use alloy_signer_gcp::{GcpKeyRingRef, GcpSigner, KeySpecifier};
use gadget_crypto::k256_crypto::{K256Ecdsa, K256Signature, K256VerifyingKey};
use gadget_std::collections::BTreeMap;
use gcloud_sdk::{
    google::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient, GoogleApi,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GcpKeyConfig {
    pub project_id: String,
    pub location: String,
    pub keyring: String,
    pub key_name: String,
    pub key_version: u64,
    pub chain_id: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GcpRemoteSignerConfig {
    pub keys: Vec<GcpKeyConfig>,
}

impl From<RemoteConfig> for GcpRemoteSignerConfig {
    fn from(config: RemoteConfig) -> Self {
        match config {
            RemoteConfig::Gcp { keys } => Self { keys },
            _ => panic!("Invalid config"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GcpKeyInstance {
    signer: GcpSigner,
    chain_id: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct GcpRemoteSigner {
    signers: BTreeMap<(String, Option<u64>), GcpKeyInstance>,
}

impl GcpRemoteSigner {
    pub async fn new(config: GcpRemoteSignerConfig) -> Result<Self> {
        let mut signers = BTreeMap::new();

        for key_config in config.keys {
            let keyring = GcpKeyRingRef::new(
                &key_config.project_id,
                &key_config.location,
                &key_config.keyring,
            );

            let client = GoogleApi::from_function(
                KeyManagementServiceClient::new,
                "https://cloudkms.googleapis.com",
                None,
            )
            .await?;

            let key_specifier =
                KeySpecifier::new(keyring, &key_config.key_name, key_config.key_version);

            let signer = GcpSigner::new(client, key_specifier, key_config.chain_id).await?;

            signers.insert(
                (key_config.key_name.clone(), key_config.chain_id),
                GcpKeyInstance {
                    signer,
                    chain_id: key_config.chain_id,
                },
            );
        }

        Ok(Self { signers })
    }
}

#[async_trait::async_trait]
impl EcdsaRemoteSigner<K256Ecdsa> for GcpRemoteSigner {
    type Public = K256VerifyingKey;
    type Signature = K256Signature;
    type KeyId = String;
    type Config = GcpRemoteSignerConfig;

    async fn build(config: RemoteConfig) -> Result<Self> {
        Self::new(config.into()).await
    }

    async fn get_public_key(
        &self,
        key_id: &Self::KeyId,
        chain_id: Option<u64>,
    ) -> Result<Self::Public> {
        // Find signer for the given key ID
        let signer = self
            .signers
            .get(&(key_id.clone(), chain_id))
            .ok_or_else(|| Error::Other(format!("No signer found for key ID {:?}", key_id)))?;

        Ok(K256VerifyingKey(
            signer
                .signer
                .get_pubkey()
                .await
                .map_err(|e| Error::RemoteKeyFetchFailed(e.to_string()))?,
        ))
    }

    async fn iter_public_keys(&self, chain_id: Option<u64>) -> Result<Vec<Self::Public>> {
        let mut public_keys = Vec::new();
        for ((_, _), signer) in &self.signers {
            // Skip if chain_id is Some and doesn't match
            if let Some(chain_id) = chain_id {
                if signer.chain_id != Some(chain_id) {
                    continue;
                }
            }

            let pk = signer
                .signer
                .get_pubkey()
                .await
                .map_err(|e| Error::RemoteKeyFetchFailed(e.to_string()))?;
            public_keys.push(K256VerifyingKey(pk));
        }
        Ok(public_keys)
    }

    async fn get_key_id_from_public_key(
        &self,
        public_key: &Self::Public,
        chain_id: Option<u64>,
    ) -> Result<Self::KeyId> {
        for ((key_id, _), signer) in &self.signers {
            // Skip if chain_id is Some and doesn't match
            if let Some(chain_id) = chain_id {
                if signer.chain_id != Some(chain_id) {
                    continue;
                }
            }

            let pk = signer
                .signer
                .get_pubkey()
                .await
                .map_err(|e| Error::RemoteKeyFetchFailed(e.to_string()))?;

            if pk == public_key.0 {
                return Ok(key_id.clone());
            }
        }

        Err(Error::KeyNotFound)
    }

    async fn sign_message_with_key_id(
        &self,
        message: &[u8],
        key_id: &Self::KeyId,
        chain_id: Option<u64>,
    ) -> Result<Self::Signature> {
        let digest = keccak256(message);

        // Find signer for the given key ID
        let signer = self
            .signers
            .get(&(key_id.clone(), chain_id))
            .ok_or_else(|| Error::Other(format!("No signer found for key ID {:?}", key_id)))?;

        Ok(K256Signature(
            signer
                .signer
                .sign_digest(&digest)
                .await
                .map_err(|e| Error::SignatureFailed(e.to_string()))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::signature::Verifier;

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_gcp_signer() {
        let config = GcpRemoteSignerConfig {
            keys: vec![GcpKeyConfig {
                project_id: gadget_std::env::var("GOOGLE_PROJECT_ID").expect("GOOGLE_PROJECT_ID not set"),
                location: gadget_std::env::var("GOOGLE_LOCATION").expect("GOOGLE_LOCATION not set"),
                keyring: gadget_std::env::var("GOOGLE_KEYRING").expect("GOOGLE_KEYRING not set"),
                key_name: gadget_std::env::var("GOOGLE_KEY_NAME").expect("GOOGLE_KEY_NAME not set"),
                key_version: 1,
                chain_id: Some(1),
            }],
        };

        let signer = GcpRemoteSigner::new(config).await.unwrap();
        let message = b"test message";

        // Get first signer's key name
        let ((key_name, _), _) = signer.signers.iter().next().unwrap();
        let key_id = key_name.clone();

        let signature = signer
            .sign_message_with_key_id(message, &key_id, Some(1))
            .await
            .unwrap();
        let pk = signer.get_public_key(&key_id, Some(1)).await.unwrap();

        assert!(pk.0.verify(message, &signature.0).is_ok());
    }
}
