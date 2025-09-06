// src/crypto.rs
use ed25519_dalek::{Signature, Verifier, VerifyingKey as PublicKey};
use hex;

/// Verify ed25519 signature where public key and signature are hex strings.
/// Returns true on successful verification, false on error/invalid lengths.
pub fn verify_ed25519_hex(pubkey_hex: &str, sig_hex: &str, message: &[u8]) -> bool {
    let pk_bytes = match hex::decode(pubkey_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let sig_bytes = match hex::decode(sig_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let pubkey_array: [u8; 32] = match pk_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => return false,
    };
    let pubkey = match PublicKey::from_bytes(&pubkey_array) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let sig_array: [u8; 64] = match sig_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => return false,
    };
    let signature = Signature::from_bytes(&sig_array);

    pubkey.verify(message, &signature).is_ok()
}
