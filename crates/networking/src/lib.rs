#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod gossip;
pub mod handlers;
pub mod messaging;
pub mod networking;
pub mod round_based_compat;

pub use round_based;

pub mod setup;

use gadget_std::string::String;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Gossip error: {0}")]
    GossipError(String),

    #[error("Messaging error: {0}")]
    MessagingError(String),

    #[error("Round based error: {0}")]
    RoundBasedError(String),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub use key_types::*;

#[cfg(all(
    feature = "sp-core-ecdsa",
    not(feature = "sp-core-sr25519"),
    not(feature = "sp-core-ed25519")
))]
pub(crate) mod key_types {
    pub use gadget_crypto::sp_core_crypto::{
        SpEcdsa as CryptoKeyCurve, SpEcdsaPair as CryptoKeyPair, SpEcdsaPublic as CryptoPublicKey,
        SpEcdsaSignature as CryptoSignature,
    };
}

#[cfg(all(
    feature = "sp-core-sr25519",
    not(feature = "sp-core-ecdsa"),
    not(feature = "sp-core-ed25519")
))]
pub(crate) mod key_types {
    pub use gadget_crypto::sp_core_crypto::{
        SpSr25519 as CryptoKeyCurve, SpSr25519Pair as CryptoKeyPair,
        SpSr25519Public as CryptoPublicKey, SpSr25519Signature as CryptoSignature,
    };
}

#[cfg(all(
    feature = "sp-core-ed25519",
    not(feature = "sp-core-ecdsa"),
    not(feature = "sp-core-sr25519")
))]
pub(crate) mod key_types {
    pub use gadget_crypto::sp_core_crypto::{
        SpEd25519 as CryptoKeyCurve, SpEd25519Pair as CryptoKeyPair,
        SpEd25519Public as CryptoPublicKey, SpEd25519Signature as CryptoSignature,
    };
}

#[cfg(all(
    not(feature = "sp-core-ecdsa"),
    not(feature = "sp-core-sr25519"),
    not(feature = "sp-core-ed25519")
))]
pub(crate) mod key_types {
    // Default to k256 ECDSA implementation
    pub use gadget_crypto::k256_crypto::{
        K256Ecdsa as CryptoKeyCurve, K256Signature as CryptoSignature,
        K256SigningKey as CryptoKeyPair, K256VerifyingKey as CryptoPublicKey,
    };
}

// Compile-time assertion to ensure only one feature is enabled
#[cfg(any(
    all(feature = "sp-core-ecdsa", feature = "sp-core-sr25519"),
    all(feature = "sp-core-ecdsa", feature = "sp-core-ed25519"),
    all(feature = "sp-core-sr25519", feature = "sp-core-ed25519")
))]
compile_error!(
    "Only one of 'sp-core-ecdsa', 'sp-core-sr25519', or 'sp-core-ed25519' features can be enabled at a time"
);
