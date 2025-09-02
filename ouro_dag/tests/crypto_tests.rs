// tests/crypto_tests.rs
// Deterministic ed25519 sign/verify test that avoids RNG version incompatibilities
use ed25519_dalek::{Keypair, Signer, PublicKey, SecretKey};
use hex;

#[test]
fn ed25519_sign_verify_hex_path() {
    // deterministic secret (NOT for production keys) — used only for test vectors
    let secret_bytes = [1u8; 32];
    let secret = SecretKey::from_bytes(&secret_bytes).expect("valid secret key bytes");

    // derive public key deterministically from secret
    let public = PublicKey::from(&secret);

    // build keypair bytes = secret || public
    let mut keypair_bytes = [0u8; 64];
    keypair_bytes[..32].copy_from_slice(&secret.to_bytes());
    keypair_bytes[32..].copy_from_slice(&public.to_bytes());

    let keypair = Keypair::from_bytes(&keypair_bytes).expect("valid keypair bytes");

    let message = b"determinism-test-message";
    let sig = keypair.sign(message);

    // verify using ed25519-dalek API (sanity)
    assert!(keypair.verify(message, &sig).is_ok());

    // hex encode as your API uses
    let pubhex = hex::encode(public.to_bytes());
    let sighex = hex::encode(sig.to_bytes());

    // call your library's verify helper
    assert!(ouro_dag::crypto::verify_ed25519_hex(&pubhex, &sighex, message));
}
