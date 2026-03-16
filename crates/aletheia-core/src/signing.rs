use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use crate::error::{AletheiaError, Result};

/// Generate a new Ed25519 keypair.
///
/// Returns `(signing_key_bytes, verifying_key_bytes)`, each 32 bytes.
pub fn generate_keypair() -> ([u8; 32], [u8; 32]) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (signing_key.to_bytes(), verifying_key.to_bytes())
}

/// Sign a 32-byte message with the provided Ed25519 signing key.
///
/// Returns a 64-byte signature.
pub fn sign(signing_key: &[u8; 32], message: &[u8; 32]) -> Result<[u8; 64]> {
    let key = SigningKey::from_bytes(signing_key);
    let sig: Signature = key.sign(message.as_slice());
    Ok(sig.to_bytes())
}

/// Verify an Ed25519 signature over a 32-byte message.
///
/// Returns `Ok(())` on success, or `AletheiaError::InvalidSignature` on failure.
pub fn verify(verifying_key: &[u8; 32], message: &[u8; 32], signature: &[u8; 64]) -> Result<()> {
    let key = VerifyingKey::from_bytes(verifying_key)
        .map_err(|e| AletheiaError::SigningError(e.to_string()))?;
    let sig = Signature::from_bytes(signature);
    key.verify(message.as_slice(), &sig).map_err(|_| AletheiaError::InvalidSignature {
        signer: hex::encode(verifying_key),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keygen_produces_valid_pair() {
        let (sk, vk) = generate_keypair();
        assert_eq!(sk.len(), 32);
        assert_eq!(vk.len(), 32);
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let (sk, vk) = generate_keypair();
        let message = [0x42u8; 32];
        let sig = sign(&sk, &message).expect("sign");
        verify(&vk, &message, &sig).expect("verify");
    }

    #[test]
    fn tampered_message_fails() {
        let (sk, vk) = generate_keypair();
        let message = [0x42u8; 32];
        let sig = sign(&sk, &message).expect("sign");

        let mut tampered = message;
        tampered[0] ^= 0xff;
        assert!(verify(&vk, &tampered, &sig).is_err());
    }

    #[test]
    fn tampered_signature_fails() {
        let (sk, vk) = generate_keypair();
        let message = [0x42u8; 32];
        let mut sig = sign(&sk, &message).expect("sign");

        // Flip a bit in the signature.
        sig[0] ^= 0x01;
        assert!(verify(&vk, &message, &sig).is_err());
    }

    #[test]
    fn wrong_key_fails() {
        let (sk, _vk) = generate_keypair();
        let (_sk2, vk2) = generate_keypair();
        let message = [0x42u8; 32];
        let sig = sign(&sk, &message).expect("sign");

        assert!(verify(&vk2, &message, &sig).is_err());
    }

    #[test]
    fn hex_roundtrip() {
        let (sk, vk) = generate_keypair();
        let sk_hex = hex::encode(sk);
        let vk_hex = hex::encode(vk);

        let sk_decoded: [u8; 32] = hex::decode(&sk_hex)
            .expect("decode sk")
            .try_into()
            .expect("32 bytes");
        let vk_decoded: [u8; 32] = hex::decode(&vk_hex)
            .expect("decode vk")
            .try_into()
            .expect("32 bytes");

        assert_eq!(sk, sk_decoded);
        assert_eq!(vk, vk_decoded);
    }
}
