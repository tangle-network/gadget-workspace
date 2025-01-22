use super::{EcdsaRemoteSigner, RemoteConfig};
use crate::error::{Error, Result};
use alloy_primitives::{Address, PrimitiveSignature};
use alloy_signer::Signer;
use alloy_signer_ledger::{HDPath, LedgerSigner};
use gadget_crypto::k256::{K256Ecdsa, K256VerifyingKey};
use gadget_std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct HDPathWrapper(pub HDPath);

impl Default for HDPathWrapper {
    fn default() -> Self {
        HDPathWrapper(HDPath::LedgerLive(0))
    }
}

impl Serialize for HDPathWrapper {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            HDPath::LedgerLive(index) => {
                serializer.serialize_str(&format!("m/44'/60'/{index}'/0/0"))
            }
            HDPath::Legacy(index) => serializer.serialize_str(&format!("m/44'/60'/0'/{index}")),
            HDPath::Other(path) => serializer.serialize_str(path),
        }
    }
}

impl<'de> Deserialize<'de> for HDPathWrapper {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;

        let hd_path = if path.starts_with("m/44'/60'/") && path.ends_with("/0/0") {
            // LedgerLive format
            let parts: Vec<&str> = path.split('/').collect();
            if let Ok(index) = parts[3].trim_end_matches("'").parse() {
                HDPath::LedgerLive(index)
            } else {
                HDPath::Other(path)
            }
        } else if path.starts_with("m/44'/60'/0'/") {
            // Legacy format
            let parts: Vec<&str> = path.split('/').collect();
            if let Ok(index) = parts[4].parse() {
                HDPath::Legacy(index)
            } else {
                HDPath::Other(path)
            }
        } else {
            HDPath::Other(path)
        };

        Ok(HDPathWrapper(hd_path))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerKeyConfig {
    pub hd_path: HDPathWrapper,
    pub chain_id: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerRemoteSignerConfig {
    pub keys: Vec<LedgerKeyConfig>,
}

impl From<RemoteConfig> for LedgerRemoteSignerConfig {
    fn from(config: RemoteConfig) -> Self {
        match config {
            RemoteConfig::Ledger { keys } => Self { keys },
            _ => panic!("Invalid config type"),
        }
    }
}

#[derive(Debug)]
pub struct LedgerKeyInstance {
    signer: LedgerSigner,
    chain_id: Option<u64>,
}

#[derive(Debug)]
pub struct LedgerRemoteSigner {
    signers: BTreeMap<(Address, Option<u64>), LedgerKeyInstance>,
}

impl LedgerRemoteSigner {
    pub async fn new(config: LedgerRemoteSignerConfig) -> Result<Self> {
        let mut signers = BTreeMap::new();

        for key_config in config.keys {
            let signer = LedgerSigner::new(key_config.hd_path.0, key_config.chain_id).await?;

            let address = signer.get_address().await?;

            signers.insert(
                (address, key_config.chain_id),
                LedgerKeyInstance {
                    signer,
                    chain_id: key_config.chain_id,
                },
            );
        }

        Ok(Self { signers })
    }

    pub fn get_signer_for_chain(&self, chain_id: Option<u64>) -> Result<&LedgerKeyInstance> {
        self.signers
            .iter()
            .find(|(_, s)| s.chain_id == chain_id)
            .map(|(_, s)| s)
            .ok_or_else(|| Error::Other("No signer found for chain ID".to_string()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub struct AddressWrapper(pub Address);
impl From<K256VerifyingKey> for AddressWrapper {
    fn from(key: K256VerifyingKey) -> Self {
        Self(Address::from_public_key(&key.0))
    }
}

#[async_trait::async_trait]
impl EcdsaRemoteSigner<K256Ecdsa> for LedgerRemoteSigner {
    type Public = AddressWrapper;
    type Signature = PrimitiveSignature;
    type KeyId = Self::Public;
    type Config = LedgerRemoteSignerConfig;

    async fn build(config: RemoteConfig) -> Result<Self> {
        Self::new(config.into()).await
    }

    async fn get_public_key(
        &self,
        key_id: &Self::KeyId,
        chain_id: Option<u64>,
    ) -> Result<Self::Public> {
        let signer = self
            .signers
            .get(&(key_id.0, chain_id))
            .ok_or_else(|| Error::Other(format!("No signer found for key ID {:?}", key_id.0)))?;

        let address = signer
            .signer
            .get_address()
            .await
            .map_err(|e| Error::RemoteKeyFetchFailed(e.to_string()))?;

        Ok(AddressWrapper(address))
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

            let address_check = signer
                .signer
                .get_address()
                .await
                .map_err(|e| Error::RemoteKeyFetchFailed(e.to_string()))?;
            public_keys.push(AddressWrapper(address_check));
        }
        Ok(public_keys)
    }

    async fn get_key_id_from_public_key(
        &self,
        address: &Self::Public,
        chain_id: Option<u64>,
    ) -> Result<Self::KeyId> {
        for ((signer_address, _), signer) in &self.signers {
            // Skip if chain_id is Some and doesn't match
            if let Some(chain_id) = chain_id {
                if signer.chain_id != Some(chain_id) {
                    continue;
                }
            }

            let signer_address_check = signer
                .signer
                .get_address()
                .await
                .map_err(|e| Error::RemoteKeyFetchFailed(e.to_string()))?;
            if signer_address_check == address.0 {
                return Ok(AddressWrapper(*signer_address));
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
        let signer = self
            .signers
            .get(&(key_id.0, chain_id))
            .ok_or_else(|| Error::Other(format!("No signer found for key ID {:?}", key_id.0)))?;

        let sig = signer
            .signer
            .sign_message(message)
            .await
            .map_err(|e| Error::SignatureFailed(e.to_string()))?;

        Ok(sig.with_parity(sig.v()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires connected Ledger device
    async fn test_ledger_signer() {
        let config = LedgerRemoteSignerConfig {
            keys: vec![LedgerKeyConfig {
                hd_path: HDPathWrapper(HDPath::LedgerLive(0)),
                chain_id: Some(1),
            }],
        };

        let signer = LedgerRemoteSigner::new(config).await.unwrap();
        let message = b"test message";

        // Get first signer's address
        let ((address, _), _) = signer.signers.iter().next().unwrap();
        let key_id = AddressWrapper(*address);

        let signature = signer
            .sign_message_with_key_id(message, &key_id, Some(1))
            .await
            .unwrap();
        let address = signer.get_public_key(&key_id, Some(1)).await.unwrap();

        assert_eq!(
            signature.recover_address_from_msg(message).unwrap(),
            address.0
        );
    }
}
