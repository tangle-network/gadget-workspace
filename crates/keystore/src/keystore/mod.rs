pub mod backends;
use backends::Backend;
use backends::BackendConfig;
cfg_remote! {
    use backends::remote::RemoteEntry;
}

mod config;
pub use config::KeystoreConfig;

use crate::error::{Error, Result};
use crate::key_types::{KeyType, KeyTypeId};
#[cfg(feature = "std")]
use crate::storage::FileStorage;
use crate::storage::{InMemoryStorage, RawStorage};
use gadget_std::{boxed::Box, cmp, collections::BTreeMap, vec::Vec};
use serde::de::DeserializeOwned;

/// Represents a storage backend with its priority
pub struct LocalStorageEntry {
    storage: Box<dyn RawStorage>,
    priority: u8,
}

pub struct Keystore {
    storages: BTreeMap<KeyTypeId, Vec<LocalStorageEntry>>,
    #[cfg(any(
        feature = "aws-signer",
        feature = "gcp-signer",
        feature = "ledger-browser",
        feature = "ledger-node"
    ))]
    remotes: BTreeMap<KeyTypeId, Vec<RemoteEntry>>,
}

impl Keystore {
    pub fn new(config: KeystoreConfig) -> Result<Self> {
        let config = config.finalize();

        let mut keystore = Self {
            storages: BTreeMap::new(),
            #[cfg(any(
                feature = "aws-signer",
                feature = "gcp-signer",
                feature = "ledger-browser",
                feature = "ledger-node"
            ))]
            remotes: BTreeMap::new(),
        };

        if config.in_memory {
            for key_type in KeyTypeId::ENABLED {
                keystore.register_storage(
                    *key_type,
                    BackendConfig::Local(Box::new(InMemoryStorage::new())),
                    0,
                )?;
            }
        }

        #[cfg(feature = "std")]
        if let Some(fs_root) = config.fs_root {
            for key_type in KeyTypeId::ENABLED {
                keystore.register_storage(
                    *key_type,
                    BackendConfig::Local(Box::new(FileStorage::new(fs_root.as_path())?)),
                    0,
                )?;
            }
        }

        #[cfg(any(
            feature = "aws-signer",
            feature = "gcp-signer",
            feature = "ledger-browser",
            feature = "ledger-node"
        ))]
        for remote_config in config.remote_configs {
            for key_type in KeyTypeId::ENABLED {
                keystore.register_storage(
                    *key_type,
                    BackendConfig::Remote(remote_config.clone()),
                    0,
                )?;
            }
        }

        Ok(keystore)
    }

    /// Register a storage backend for a key type with priority
    fn register_storage(
        &mut self,
        key_type_id: KeyTypeId,
        storage: BackendConfig,
        priority: u8,
    ) -> Result<()> {
        match storage {
            BackendConfig::Local(storage) => {
                let entry = LocalStorageEntry { storage, priority };
                let backends = self.storages.entry(key_type_id).or_default();
                backends.push(entry);
                backends.sort_by_key(|e| cmp::Reverse(e.priority));
            }
            #[cfg(any(
                feature = "aws-signer",
                feature = "gcp-signer",
                feature = "ledger-browser",
                feature = "ledger-node"
            ))]
            BackendConfig::Remote(_config) => return Err(Error::StorageNotSupported),
        }
        Ok(())
    }
}

impl Backend for Keystore {
    /// Generate a new key pair from random seed
    fn generate<T: KeyType>(&self, seed: Option<&[u8]>) -> Result<T::Public>
    where
        T::Public: DeserializeOwned,
        T::Secret: DeserializeOwned,
    {
        let backends = self.get_storage_backends::<T>()?;
        let secret = T::generate_with_seed(seed)?;
        let public = T::public_from_secret(&secret);

        // Store in all available storage backends
        for entry in backends {
            entry.storage.store_raw(
                T::key_type_id(),
                serde_json::to_vec(&public)?,
                serde_json::to_vec(&secret)?,
            )?;
        }

        Ok(public)
    }

    /// Generate a key pair from a string seed
    fn generate_from_string<T: KeyType>(&self, seed_str: &str) -> Result<T::Public>
    where
        T::Public: DeserializeOwned,
        T::Secret: DeserializeOwned,
    {
        let seed = blake3::hash(seed_str.as_bytes()).as_bytes().to_vec();
        self.generate::<T>(Some(&seed))
    }

    /// Sign a message using a local key
    fn sign_with_local<T: KeyType>(&self, public: &T::Public, msg: &[u8]) -> Result<T::Signature>
    where
        T::Public: DeserializeOwned,
        T::Secret: DeserializeOwned,
    {
        let secret = self.get_secret::<T>(public)?;
        T::sign_with_secret(&mut secret.clone(), msg)
    }

    /// List all public keys of a given type from storages
    fn list_local<T: KeyType>(&self) -> Result<Vec<T::Public>>
    where
        T::Public: DeserializeOwned,
    {
        let mut keys = Vec::new();
        let key_type = T::key_type_id();

        if let Some(backends) = self.storages.get(&key_type) {
            for entry in backends {
                let mut backend_keys: Vec<T::Public> = entry
                    .storage
                    .list_raw(T::key_type_id())
                    .filter_map(|bytes| serde_json::from_slice(&bytes).ok())
                    .collect();
                keys.append(&mut backend_keys);
            }
        }

        keys.sort_unstable();
        keys.dedup();
        Ok(keys)
    }

    fn get_public_key_local<T: KeyType>(&self, key_id: &str) -> Result<T::Public>
    where
        T::Public: DeserializeOwned,
    {
        // First check local storage
        let storages = self
            .storages
            .get(&T::key_type_id())
            .ok_or(Error::KeyTypeNotSupported)?;

        for entry in storages {
            if let Some(bytes) = entry.storage.load_raw(T::key_type_id(), key_id.into())? {
                let public: T::Public = serde_json::from_slice(&bytes)?;
                return Ok(public);
            }
        }

        Err(Error::KeyNotFound)
    }

    fn contains_local<T: KeyType>(&self, public: &T::Public) -> Result<bool> {
        let public_bytes = serde_json::to_vec(public)?;
        let storages = self
            .storages
            .get(&T::key_type_id())
            .ok_or(Error::KeyTypeNotSupported)?;

        for entry in storages {
            if entry
                .storage
                .contains_raw(T::key_type_id(), public_bytes.clone())
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn remove<T: KeyType>(&self, public: &T::Public) -> Result<()>
    where
        T::Public: DeserializeOwned,
    {
        let public_bytes = serde_json::to_vec(public)?;
        let storages = self
            .storages
            .get(&T::key_type_id())
            .ok_or(Error::KeyTypeNotSupported)?;

        for entry in storages {
            entry
                .storage
                .remove_raw(T::key_type_id(), public_bytes.clone())?;
        }

        Ok(())
    }

    fn get_secret<T: KeyType>(&self, public: &T::Public) -> Result<T::Secret>
    where
        T::Public: DeserializeOwned,
        T::Secret: DeserializeOwned,
    {
        let storages = self
            .storages
            .get(&T::key_type_id())
            .ok_or(Error::KeyTypeNotSupported)?;

        let public_bytes = serde_json::to_vec(public)?;
        for entry in storages {
            if let Some(bytes) = entry
                .storage
                .load_raw(T::key_type_id(), public_bytes.clone())?
            {
                let secret: T::Secret = serde_json::from_slice(&bytes)?;
                return Ok(secret);
            }
        }

        Err(Error::KeyNotFound)
    }

    // Helper methods
    fn get_storage_backends<T: KeyType>(&self) -> Result<&[LocalStorageEntry]> {
        self.storages
            .get(&T::key_type_id())
            .map(|v| v.as_slice())
            .ok_or(Error::KeyTypeNotSupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_types::k256_ecdsa::K256Ecdsa;

    #[test]
    fn test_generate_from_string() -> Result<()> {
        let keystore = Keystore::new(KeystoreConfig::new())?;

        let seed = "test seed string";
        let public1 = keystore.generate_from_string::<K256Ecdsa>(seed)?;
        let public2 = keystore.generate_from_string::<K256Ecdsa>(seed)?;

        // Same seed should generate same key
        assert_eq!(public1, public2);

        // Different seeds should generate different keys
        let public3 = keystore.generate_from_string::<K256Ecdsa>("different seed")?;
        assert_ne!(public1, public3);

        Ok(())
    }

    #[tokio::test]
    async fn test_local_operations() -> Result<()> {
        let keystore = Keystore::new(KeystoreConfig::new())?;

        // Generate and test local key
        let public = keystore.generate::<K256Ecdsa>(None)?;
        let message = b"test message";
        let signature = keystore.sign_with_local::<K256Ecdsa>(&public, message)?;
        assert!(K256Ecdsa::verify(&public, message, &signature));

        // List local keys
        let local_keys = keystore.list_local::<K256Ecdsa>()?;
        assert_eq!(local_keys.len(), 1);
        assert_eq!(local_keys[0], public);

        Ok(())
    }
}
