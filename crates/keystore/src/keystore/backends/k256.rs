use super::*;
use crate::error::{Error, Result};
use crate::Keystore;
use gadget_crypto::k256::{K256Ecdsa, K256Signature, K256SigningKey, K256VerifyingKey};
use gadget_crypto::{KeyEncoding, KeyTypeId};

#[async_trait::async_trait]
pub trait K256EcdsaBackend: Send + Sync {
    /// Generate a new K256 ECDSA key pair from seed
    fn k256_ecdsa_generate_new(&self, seed: Option<&[u8]>) -> Result<K256VerifyingKey>;

    /// Generate a K256 ECDSA key pair from a string seed
    fn k256_ecdsa_generate_from_string(&self, secret: &str) -> Result<K256VerifyingKey>;

    /// Sign a message using K256 ECDSA key
    fn k256_ecdsa_sign(
        &self,
        public: &K256VerifyingKey,
        msg: &[u8; 32],
    ) -> Result<Option<K256Signature>>;

    /// Get the secret key for a K256 ECDSA public key
    fn expose_k256_ecdsa_secret(&self, public: &K256VerifyingKey)
        -> Result<Option<K256SigningKey>>;

    /// Iterate over all K256 ECDSA public keys
    fn iter_k256_ecdsa(&self) -> Box<dyn Iterator<Item = K256VerifyingKey> + '_>;
}

impl K256EcdsaBackend for Keystore {
    fn k256_ecdsa_generate_new(&self, seed: Option<&[u8]>) -> Result<K256VerifyingKey> {
        let secret =
            K256Ecdsa::generate_with_seed(seed).map_err(|e| Error::Other(e.to_string()))?;
        insert_k256_ecdsa_key(self, &secret)
    }

    fn k256_ecdsa_generate_from_string(&self, seed_string: &str) -> Result<K256VerifyingKey> {
        let secret = K256Ecdsa::generate_with_string(seed_string.to_string())
            .map_err(|e| Error::Other(e.to_string()))?;
        insert_k256_ecdsa_key(self, &secret)
    }

    fn k256_ecdsa_sign(
        &self,
        public: &K256VerifyingKey,
        msg: &[u8; 32],
    ) -> Result<Option<K256Signature>> {
        if let Some(mut secret) = self.expose_k256_ecdsa_secret(public)? {
            Ok(Some(
                K256Ecdsa::sign_with_secret(&mut secret, msg)
                    .map_err(|e| Error::Other(e.to_string()))?,
            ))
        } else {
            Ok(None)
        }
    }

    fn expose_k256_ecdsa_secret(
        &self,
        public: &K256VerifyingKey,
    ) -> Result<Option<K256SigningKey>> {
        let public_bytes = public.to_bytes();

        if let Some(storages) = self.storages.get(&KeyTypeId::K256) {
            for entry in storages {
                if let Some(secret_bytes) = entry
                    .storage
                    .load_secret_raw(KeyTypeId::K256, public_bytes.clone())?
                {
                    return Ok(Some(K256SigningKey::from_bytes(&secret_bytes)?));
                }
            }
        }

        Ok(None)
    }

    fn iter_k256_ecdsa(&self) -> Box<dyn Iterator<Item = K256VerifyingKey> + '_> {
        let Some(storages) = self.storages.get(&KeyTypeId::K256) else {
            return Box::new(std::iter::empty());
        };

        let mut keys = Vec::new();
        for entry in storages {
            let mut storage_keys = entry
                .storage
                .list_raw(KeyTypeId::K256)
                .filter_map(|bytes| K256VerifyingKey::from_bytes(&bytes).ok())
                .collect::<Vec<_>>();
            keys.append(&mut storage_keys);
        }
        Box::new(keys.into_iter())
    }
}

fn insert_k256_ecdsa_key(keystore: &Keystore, secret: &K256SigningKey) -> Result<K256VerifyingKey> {
    let public = K256Ecdsa::public_from_secret(secret);
    let public_bytes = public.to_bytes();
    let secret_bytes = secret.to_bytes();

    if let Some(storages) = keystore.storages.get(&KeyTypeId::K256) {
        for entry in storages {
            entry
                .storage
                .store_raw(KeyTypeId::K256, public_bytes.clone(), secret_bytes.clone())?;
        }
    }

    Ok(public)
}
