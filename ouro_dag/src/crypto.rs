// src/crypto.rs
use ed25519_dalek::{PublicKey, Signature, Verifier};
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

    if pk_bytes.len() != 32 || sig_bytes.len() != 64 {
        return false;
    }

    let pubkey = match PublicKey::from_bytes(&pk_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let signature = match Signature::from_bytes(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    pubkey.verify(message, &signature).is_ok()
}
