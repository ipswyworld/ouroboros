// tests/crypto_tests.rs
// Deterministic ed25519 sign/verify test that avoids RNG version incompatibilities
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use hex;

#[test]
fn ed25519_sign_verify_hex_path() {
    // deterministic secret (NOT for production keys) — used only for test vectors
    let secret_bytes = [1u8; 32];
    let signing_key = SigningKey::from_bytes(&secret_bytes);

    // derive public key deterministically from secret
    let verifying_key: VerifyingKey = (&signing_key).into();

    let message = b"determinism-test-message";
    let sig = signing_key.sign(message);

    // verify using ed25519-dalek API (sanity)
    assert!(signing_key.verify(message, &sig).is_ok());

    // hex encode as your API uses
    let pubhex = hex::encode(verifying_key.to_bytes());
    let sighex = hex::encode(sig.to_bytes());

    // call your library's verify helper
    assert!(ouro_dag::crypto::verify_ed25519_hex(&pubhex, &sighex, message));
}
