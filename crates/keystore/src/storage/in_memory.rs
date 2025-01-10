use super::RawStorage;
use crate::error::Result;
use gadget_crypto::KeyTypeId;
use gadget_std::{boxed::Box, collections::BTreeMap, vec::Vec};
use parking_lot::RwLock;

type StorageMap = BTreeMap<KeyTypeId, BTreeMap<Vec<u8>, Vec<u8>>>;

/// A memory-backed local storage
pub struct InMemoryStorage {
    data: RwLock<StorageMap>,
}

impl InMemoryStorage {
    /// Create a new `InMemoryStorage`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gadget_keystore::backends::{Backend, BackendConfig};
    /// use gadget_keystore::crypto::k256::K256Ecdsa;
    /// use gadget_keystore::crypto::IntoCryptoError;
    /// use gadget_keystore::crypto::KeyType;
    /// use gadget_keystore::storage::{InMemoryStorage, TypedStorage};
    /// use gadget_keystore::Keystore;
    ///
    /// # fn main() -> gadget_keystore::Result<()> {
    /// // Create the storage
    /// let storage = InMemoryStorage::new();
    /// let storage = TypedStorage::new(storage);
    ///
    /// // Generate a key pair
    /// let secret = K256Ecdsa::generate_with_seed(None).unwrap();
    /// let public = K256Ecdsa::public_from_secret(&secret);
    ///
    /// // Start storing
    /// storage.store::<K256Ecdsa>(&public, &secret)?;
    /// # Ok(()) }
    /// ```
    pub fn new() -> Self {
        Self {
            data: RwLock::new(BTreeMap::new()),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl RawStorage for InMemoryStorage {
    fn store_raw(
        &self,
        type_id: KeyTypeId,
        public_bytes: Vec<u8>,
        secret_bytes: Vec<u8>,
    ) -> Result<()> {
        let mut data = self.data.write();
        let type_map = data.entry(type_id).or_default();
        type_map.insert(public_bytes, secret_bytes);
        Ok(())
    }

    fn load_secret_raw(
        &self,
        type_id: KeyTypeId,
        public_bytes: Vec<u8>,
    ) -> Result<Option<Box<[u8]>>> {
        let data = self.data.read();
        Ok(data
            .get(&type_id)
            .and_then(|type_map| type_map.get(&public_bytes[..]))
            .map(|v| v.clone().into_boxed_slice()))
    }

    fn remove_raw(&self, type_id: KeyTypeId, public_bytes: Vec<u8>) -> Result<()> {
        let mut data = self.data.write();
        if let Some(type_map) = data.get_mut(&type_id) {
            type_map.remove(&public_bytes[..]);
        }
        Ok(())
    }

    fn contains_raw(&self, type_id: KeyTypeId, public_bytes: Vec<u8>) -> bool {
        let data = self.data.read();
        data.get(&type_id)
            .map(|type_map| type_map.contains_key(&public_bytes[..]))
            .unwrap_or(false)
    }

    fn list_raw(&self, type_id: KeyTypeId) -> Box<dyn Iterator<Item = Box<[u8]>> + '_> {
        let data = self.data.read();
        let keys = data
            .get(&type_id)
            .map(|type_map| {
                type_map
                    .keys()
                    .map(|k| k.clone().into_boxed_slice())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Box::new(keys.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use gadget_crypto::{k256::K256Ecdsa, IntoCryptoError, KeyType};

    use super::*;
    use crate::storage::TypedStorage;

    #[test]
    fn test_basic_operations() -> Result<()> {
        let raw_storage = InMemoryStorage::new();
        let storage = TypedStorage::new(raw_storage);

        // Generate a key pair
        let secret =
            K256Ecdsa::generate_with_seed(None).map_err(IntoCryptoError::into_crypto_error)?;
        let public = K256Ecdsa::public_from_secret(&secret);

        // Test store and load
        storage.store::<K256Ecdsa>(&public, &secret)?;
        let loaded = storage.load::<K256Ecdsa>(&public)?;
        assert_eq!(loaded.as_ref(), Some(&secret));

        // Test contains
        assert!(storage.contains::<K256Ecdsa>(&public));

        // Test list
        let keys: Vec<_> = storage.list::<K256Ecdsa>().collect();
        assert_eq!(keys.len(), 1);
        assert_eq!(&keys[0], &public);

        // Test remove
        storage.remove::<K256Ecdsa>(&public)?;
        assert!(!storage.contains::<K256Ecdsa>(&public));
        assert_eq!(storage.load::<K256Ecdsa>(&public)?, None);

        Ok(())
    }

    #[test]
    fn test_multiple_key_types() -> Result<()> {
        let raw_storage = InMemoryStorage::new();
        let storage = TypedStorage::new(raw_storage);

        // Create keys of different types
        let k256_secret =
            K256Ecdsa::generate_with_seed(None).map_err(IntoCryptoError::into_crypto_error)?;
        let k256_public = K256Ecdsa::public_from_secret(&k256_secret);

        // Store keys
        storage.store::<K256Ecdsa>(&k256_public, &k256_secret)?;

        // Verify isolation between types
        assert!(storage.contains::<K256Ecdsa>(&k256_public));

        // List should only show keys of the requested type
        assert_eq!(storage.list::<K256Ecdsa>().count(), 1);

        Ok(())
    }
}
