use clap::ValueEnum;
use color_eyre::eyre::Result;
use gadget_crypto::{
    bls_crypto::{W3fBls377, W3fBls381},
    bn254_crypto::ArkBlsBn254,
    k256_crypto::K256Ecdsa,
    sp_core_crypto::{SpBls377, SpBls381, SpEcdsa, SpEd25519, SpSr25519},
    KeyType, KeyTypeId,
};
use gadget_keystore::{Keystore, KeystoreConfig};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unknown key type: {0}")]
    UnknownKeyType(String),
    #[error("Keystore error: {0}")]
    KeystoreError(#[from] gadget_keystore::error::Error),
}

impl ValueEnum for KeyTypeId {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::SchnorrkelSr25519,
            Self::ZebraEd25519,
            Self::K256Ecdsa,
            Self::W3fBls381,
            Self::ArkBn254,
            Self::SpEcdsa,
            Self::SpEd25519,
            Self::SpSr25519,
            Self::SpBls377,
            Self::SpBls381,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::SchnorrkelSr25519 => {
                clap::builder::PossibleValue::new("sr25519").help("Schnorrkel/Ristretto x25519")
            }
            Self::ZebraEd25519 => {
                clap::builder::PossibleValue::new("ed25519").help("Edwards Curve 25519")
            }
            Self::K256Ecdsa => clap::builder::PossibleValue::new("ecdsa")
                .help("Elliptic Curve Digital Signature Algorithm"),
            Self::W3fBls381 => {
                clap::builder::PossibleValue::new("bls381").help("Boneh-Lynn-Shacham on BLS12-381")
            }
            Self::ArkBn254 => {
                clap::builder::PossibleValue::new("blsbn254").help("Boneh-Lynn-Shacham on BN254")
            }
            Self::SpEcdsa => clap::builder::PossibleValue::new("sp-ecdsa").help("Substrate ECDSA"),
            Self::SpEd25519 => {
                clap::builder::PossibleValue::new("sp-ed25519").help("Substrate Ed25519")
            }
            Self::SpSr25519 => {
                clap::builder::PossibleValue::new("sp-sr25519").help("Substrate Sr25519")
            }
            Self::SpBls377 => {
                clap::builder::PossibleValue::new("sp-bls377").help("Substrate BLS377")
            }
            Self::SpBls381 => {
                clap::builder::PossibleValue::new("sp-bls381").help("Substrate BLS381")
            }
            _ => return None,
        })
    }
}

pub fn generate_key(
    key_type: KeyTypeId,
    output: Option<PathBuf>,
    seed: Option<&[u8]>,
    show_secret: bool,
) -> Result<()> {
    // Create keystore configuration
    let mut config = KeystoreConfig::new();
    if let Some(path) = output {
        config = config.fs_root(path);
    }

    let keystore = Keystore::new(config)?;

    // Generate key based on type
    let (public, secret) = match key_type {
        KeyTypeId::SchnorrkelSr25519 => {
            let public = keystore.generate::<SpSr25519>(seed)?;
            let secret = keystore
                .expose_sr25519_secret(&public)?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to expose secret"))?;
            (
                hex::encode(public.to_raw_vec()),
                hex::encode(secret.to_raw_vec()),
            )
        }
        KeyTypeId::ZebraEd25519 => {
            let public = keystore.generate::<SpEd25519>(seed)?;
            let secret = keystore
                .expose_ed25519_secret(&public)?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to expose secret"))?;
            (
                hex::encode(public.to_raw_vec()),
                hex::encode(secret.to_raw_vec()),
            )
        }
        KeyTypeId::K256Ecdsa => {
            let public = keystore.generate::<K256Ecdsa>(seed)?;
            let secret = keystore.expose_secret::<K256Ecdsa>(&public)?;
            (
                hex::encode(public.to_bytes()),
                hex::encode(secret.to_bytes()),
            )
        }
        KeyTypeId::W3fBls381 => {
            let public = keystore.generate::<W3fBls381>(seed)?;
            let secret = keystore.expose_secret::<W3fBls381>(&public)?;
            (
                hex::encode(public.0.to_bytes()),
                hex::encode(secret.0.to_bytes()),
            )
        }
        KeyTypeId::ArkBn254 => {
            let public = keystore.generate::<ArkBlsBn254>(seed)?;
            let secret = keystore.expose_secret::<ArkBlsBn254>(&public)?;
            (
                hex::encode(public.0.to_bytes()),
                hex::encode(secret.0.to_bytes()),
            )
        }
        KeyTypeId::SpEcdsa => {
            let public = keystore.generate::<SpEcdsa>(seed)?;
            let secret = keystore
                .expose_ecdsa_secret(&public)?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to expose secret"))?;
            (
                hex::encode(public.to_raw_vec()),
                hex::encode(secret.to_raw_vec()),
            )
        }
        KeyTypeId::SpEd25519 => {
            let public = keystore.generate::<SpEd25519>(seed)?;
            let secret = keystore
                .expose_ed25519_secret(&public)?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to expose secret"))?;
            (
                hex::encode(public.to_raw_vec()),
                hex::encode(secret.to_raw_vec()),
            )
        }
        KeyTypeId::SpSr25519 => {
            let public = keystore.generate::<SpSr25519>(seed)?;
            let secret = keystore
                .expose_sr25519_secret(&public)?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to expose secret"))?;
            (
                hex::encode(public.to_raw_vec()),
                hex::encode(secret.to_raw_vec()),
            )
        }
        KeyTypeId::SpBls377 => {
            let public = keystore.generate::<SpBls377>(seed)?;
            let secret = keystore
                .expose_bls377_secret(&public)?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to expose secret"))?;
            (
                hex::encode(public.to_raw_vec()),
                hex::encode(secret.to_raw_vec()),
            )
        }
        KeyTypeId::SpBls381 => {
            let public = keystore.generate::<SpBls381>(seed)?;
            let secret = keystore
                .expose_bls381_secret(&public)?
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to expose secret"))?;
            (
                hex::encode(public.to_raw_vec()),
                hex::encode(secret.to_raw_vec()),
            )
        }
        _ => return Err(Error::UnknownKeyType(key_type.name().to_string()).into()),
    };

    eprintln!("Generated {} key:", key_type.name());
    eprintln!("Public key: {}", public);
    if show_secret || output.is_none() {
        eprintln!("Private key: {}", secret);
    }

    Ok(())
}
