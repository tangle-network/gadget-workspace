pub const CONTEXT: &[u8] = b"tangle";

macro_rules! impl_w3f_serde {
    ($name:ident, $inner:ty) => {
        #[derive(Clone)]
        pub struct $name(pub $inner);

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                to_bytes(self.0.clone()) == to_bytes(other.0.clone())
            }
        }

        impl Eq for $name {}

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<gadget_std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> gadget_std::cmp::Ordering {
                to_bytes(self.0.clone()).cmp(&to_bytes(other.0.clone()))
            }
        }

        impl gadget_std::fmt::Debug for $name {
            fn fmt(&self, f: &mut gadget_std::fmt::Formatter<'_>) -> gadget_std::fmt::Result {
                write!(f, "{:?}", to_bytes(self.0.clone()))
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(
                &self,
                serializer: S,
            ) -> core::result::Result<S::Ok, S::Error> {
                let bytes = to_bytes(self.0.clone());
                Vec::serialize(&bytes, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                // Deserialize as Vec
                let bytes = <gadget_std::vec::Vec<u8>>::deserialize(deserializer)?;

                // Convert bytes back to inner type
                let inner = from_bytes::<$inner>(&bytes);

                Ok($name(inner))
            }
        }
    };
}

macro_rules! define_bls_key {
    ($($ty:ident),+) => {
        paste::paste! {
            $(
            pub mod [<$ty:lower>] {
                use crate::error::{BlsError, Result};
                use crate::{from_bytes, to_bytes};
                use gadget_crypto_core::{KeyType, KeyTypeId};
                use gadget_std::{UniformRand, string::{String, ToString}};
                use w3f_bls::{Message, PublicKey, SecretKey, SerializableToBytes, Signature, [<Tiny $ty:upper>]};

                #[doc = $ty:upper]
                /// key type
                pub struct [<W3f $ty>];

                impl_w3f_serde!(Public, PublicKey<[<Tiny $ty:upper>]>);
                impl_w3f_serde!(Secret, SecretKey<[<Tiny $ty:upper>]>);
                impl_w3f_serde!([<W3f $ty Signature>], Signature<[<Tiny $ty:upper>]>);

                impl KeyType for [<W3f $ty>] {
                    type Public = Public;
                    type Secret = Secret;
                    type Signature = [<W3f $ty Signature>];
                    type Error = BlsError;

                    fn key_type_id() -> KeyTypeId {
                        KeyTypeId::[<W3f $ty>]
                    }

                    fn generate_with_seed(seed: Option<&[u8]>) -> Result<Self::Secret> {
                        if let Some(seed) = seed {
                            Ok(Secret(SecretKey::from_seed(seed)))
                        } else {
                            // Should only be used for testing. Pass a seed in production.
                            let mut rng = gadget_std::test_rng();
                            let rand_bytes = <[u8; 32]>::rand(&mut rng);
                            Ok(Secret(SecretKey::from_seed(&rand_bytes)))
                        }
                    }

                    fn generate_with_string(secret: String) -> Result<Self::Secret> {
                        let hex_encoded = hex::decode(secret)?;
                        let secret =
                            SecretKey::from_bytes(&hex_encoded).map_err(|e| BlsError::InvalidSeed(e.to_string()))?;
                        Ok(Secret(secret))
                    }

                    fn public_from_secret(secret: &Self::Secret) -> Self::Public {
                        Public(secret.0.into_public())
                    }

                    fn sign_with_secret(secret: &mut Self::Secret, msg: &[u8]) -> Result<Self::Signature> {
                        let mut rng = Self::get_rng();
                        let message: Message = Message::new(super::CONTEXT, msg);
                        Ok([<W3f $ty Signature>](secret.0.sign(&message, &mut rng)))
                    }

                    fn sign_with_secret_pre_hashed(
                        secret: &mut Self::Secret,
                        msg: &[u8; 32],
                    ) -> Result<Self::Signature> {
                        let mut rng = Self::get_rng();
                        let message: Message = Message::new(super::CONTEXT, msg);
                        Ok([<W3f $ty Signature>](secret.0.sign(&message, &mut rng)))
                    }

                    fn verify(public: &Self::Public, msg: &[u8], signature: &Self::Signature) -> bool {
                        let message = Message::new(super::CONTEXT, msg);
                        signature.0.verify(&message, &public.0)
                    }
                }
            }
            )+
        }
    }
}

define_bls_key!(Bls377, Bls381);
