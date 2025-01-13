use super::*;
use gadget_crypto_core::KeyType;

mod bls381_tests {
    use super::*;
    use sp_core::Pair;

    #[test]
    fn test_bls381_key_generation() {
        // Test random key generation
        let secret = SpBls381::generate_with_seed(None).unwrap();
        let public = SpBls381::public_from_secret(&secret);

        // Test generation with seed
        let seed: [u8; 32] = [1u8; 32];
        let secret_with_seed = SpBls381::generate_with_seed(Some(&seed)).unwrap();
        let public_with_seed = SpBls381::public_from_secret(&secret_with_seed);

        assert_ne!(
            secret.to_raw_vec(),
            secret_with_seed.to_raw_vec(),
            "Random and seeded keys should be different"
        );
        assert_ne!(public, public_with_seed, "Public keys should be different");
    }

    #[test]
    fn test_bls381_sign_and_verify() {
        let seed: [u8; 32] = [1u8; 32];
        let mut secret = SpBls381::generate_with_seed(Some(&seed)).unwrap();
        let public = SpBls381::public_from_secret(&secret);

        // Test normal signing
        let message = b"Hello, world!";
        let signature = SpBls381::sign_with_secret(&mut secret, message).unwrap();
        assert!(
            SpBls381::verify(&public, message, &signature),
            "Signature verification failed"
        );

        // Test pre-hashed signing
        let hashed_msg = [42u8; 32];
        let signature = SpBls381::sign_with_secret_pre_hashed(&mut secret, &hashed_msg).unwrap();

        // Verify with wrong message should fail
        let wrong_message = b"Wrong message";
        assert!(
            !SpBls381::verify(&public, wrong_message, &signature),
            "Verification should fail with wrong message"
        );
    }

    #[test]
    fn test_bls381_key_serialization() {
        let seed: [u8; 32] = [1u8; 32];
        let secret = SpBls381::generate_with_seed(Some(&seed)).unwrap();
        let public = SpBls381::public_from_secret(&secret);

        // Test signing key serialization using seed
        let serialized = serde_json::to_vec(&seed).unwrap();
        let deserialized: SpBls381Pair = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(
            secret.to_raw_vec(),
            deserialized.to_raw_vec(),
            "SigningKey serialization roundtrip failed"
        );

        // Test verifying key serialization
        let serialized = serde_json::to_string(&public).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            public, deserialized,
            "VerifyingKey serialization roundtrip failed"
        );
    }

    #[test]
    fn test_bls381_signature_serialization() {
        let seed: [u8; 32] = [1u8; 32];
        let mut secret = SpBls381::generate_with_seed(Some(&seed)).unwrap();
        let message = b"Test message";
        let signature = SpBls381::sign_with_secret(&mut secret, message).unwrap();

        // Test signature serialization
        let serialized = serde_json::to_string(&signature).unwrap();
        let deserialized: SpBls381Signature = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            signature, deserialized,
            "Signature serialization roundtrip failed"
        );
    }
}

mod bls377_tests {
    use super::*;
    use sp_core::Pair;

    #[test]
    fn test_bls377_key_generation() {
        // Test random key generation
        let secret = SpBls377::generate_with_seed(None).unwrap();
        let public = SpBls377::public_from_secret(&secret);

        // Test generation with seed
        let seed: [u8; 32] = [1u8; 32];
        let secret_with_seed = SpBls377::generate_with_seed(Some(&seed)).unwrap();
        let public_with_seed = SpBls377::public_from_secret(&secret_with_seed);

        assert_ne!(
            secret.to_raw_vec(),
            secret_with_seed.to_raw_vec(),
            "Random and seeded keys should be different"
        );
        assert_ne!(public, public_with_seed, "Public keys should be different");
    }

    #[test]
    fn test_bls377_sign_and_verify() {
        let seed: [u8; 32] = [1u8; 32];
        let mut secret = SpBls377::generate_with_seed(Some(&seed)).unwrap();
        let public = SpBls377::public_from_secret(&secret);

        // Test normal signing
        let message = b"Hello, world!";
        let signature = SpBls377::sign_with_secret(&mut secret, message).unwrap();
        assert!(
            SpBls377::verify(&public, message, &signature),
            "Signature verification failed"
        );

        // Test pre-hashed signing
        let hashed_msg = [42u8; 32];
        let signature = SpBls377::sign_with_secret_pre_hashed(&mut secret, &hashed_msg).unwrap();

        // Verify with wrong message should fail
        let wrong_message = b"Wrong message";
        assert!(
            !SpBls377::verify(&public, wrong_message, &signature),
            "Verification should fail with wrong message"
        );
    }

    #[test]
    fn test_bls377_key_serialization() {
        let seed: [u8; 32] = [1u8; 32];
        let secret = SpBls377::generate_with_seed(Some(&seed)).unwrap();
        let public = SpBls377::public_from_secret(&secret);

        // Test signing key serialization using seed
        let serialized = serde_json::to_vec(&seed).unwrap();
        let deserialized: SpBls377Pair = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(
            secret.to_raw_vec(),
            deserialized.to_raw_vec(),
            "SigningKey serialization roundtrip failed"
        );

        // Test verifying key serialization
        let serialized = serde_json::to_string(&public).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            public, deserialized,
            "VerifyingKey serialization roundtrip failed"
        );
    }

    #[test]
    fn test_bls377_signature_serialization() {
        let seed: [u8; 32] = [1u8; 32];
        let mut secret = SpBls377::generate_with_seed(Some(&seed)).unwrap();
        let message = b"Test message";
        let signature = SpBls377::sign_with_secret(&mut secret, message).unwrap();

        // Test signature serialization
        let serialized = serde_json::to_string(&signature).unwrap();
        let deserialized: SpBls377Signature = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            signature, deserialized,
            "Signature serialization roundtrip failed"
        );
    }
}
