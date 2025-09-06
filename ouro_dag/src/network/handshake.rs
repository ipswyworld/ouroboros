// src/network/handshake.rs
use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, SigningKey, Signer, VerifyingKey as PublicKey, Verifier};
use hex;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::{SinkExt, StreamExt};

/// Hello sent by client on connect
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hello {
    pub node_id: String,
    pub public_key: String, // hex
}

/// Challenge sent by server after Hello
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Challenge {
    pub nonce: String, // hex
}

/// Signature message sent by client in response to Challenge
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignatureMsg {
    pub signature: String, // hex
}

/// Peer list message (simple gossip)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerList {
    pub peers: Vec<String>,
}

/// Simple ACK response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ack {
    pub ack: bool,
}

/// Top-level Envelope message: versioned, typed payload
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Envelope {
    pub version: u8,
    pub typ: String,
    pub payload: serde_json::Value,
}

impl Envelope {
    pub fn new<T: serde::Serialize>(typ: &str, payload: &T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            version: 1,
            typ: typ.to_string(),
            payload: serde_json::to_value(payload)?,
        })
    }
}

/// Basic peer info returned after handshake
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub public_key_hex: String,
}

/// Load a SigningKey from env "NODE_KEYPAIR_HEX" if present.
/// Expect hex(64 bytes) = 32 secret + 32 public (ed25519 keypair bytes)
pub fn load_keypair_from_env() -> Option<SigningKey> {
    match std::env::var("NODE_KEYPAIR_HEX") {
        Ok(hexstr) => {
            let b = match hex::decode(hexstr.trim()) {
                Ok(b) => b,
                Err(_) => return None,
            };
            if b.len() != 64 {
                return None;
            }
            let arr: [u8; 64] = match b.try_into() {
                Ok(a) => a,
                Err(_) => return None,
            };
            SigningKey::from_keypair_bytes(&arr).ok()
        }
        Err(_) => None,
    }
}

/// Helper: generate ephemeral keypair (dev only)
pub fn generate_ephemeral_keypair() -> SigningKey {
    let mut csprng = OsRng {};
    SigningKey::generate(&mut csprng)
}

/// deterministic message id for dedupe (sha256 hex of type + canonical payload)
pub fn message_id_from_envelope(env: &Envelope) -> String {
    let mut hasher = Sha256::new();
    hasher.update(env.typ.as_bytes());
    let json_bytes = serde_json::to_vec(&env.payload).unwrap_or_default();
    hasher.update(&json_bytes);
    hex::encode(hasher.finalize())
}

/// Server-side handshake: read hello -> send challenge -> verify signature -> send peer_list
/// Returns (PeerInfo, framed transport) so caller can read further framed messages.
pub async fn server_handshake_and_upgrade(
    stream: TcpStream,
) -> Result<(PeerInfo, Framed<TcpStream, LengthDelimitedCodec>)> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

    // read hello envelope
    let frame = framed.next().await.ok_or_else(|| anyhow!("peer closed during hello"))??;
    let env: Envelope = serde_json::from_slice(&frame)?;
    if env.typ != "hello" {
        return Err(anyhow!("expected hello, got {}", env.typ));
    }
    let hello: Hello = serde_json::from_value(env.payload)?;

    // send challenge
    let mut nonce = [0u8; 16];
    OsRng.fill_bytes(&mut nonce);
    let nonce_hex = hex::encode(nonce);
    let challenge = Challenge { nonce: nonce_hex.clone() };
    let env_ch = Envelope::new("challenge", &challenge)?;
    let bytes = serde_json::to_vec(&env_ch)?;
    framed.send(bytes.into()).await?;

    // read signature
    let frame = framed.next().await.ok_or_else(|| anyhow!("peer closed during signature"))??;
    let env_sig: Envelope = serde_json::from_slice(&frame)?;
    if env_sig.typ != "signature" {
        return Err(anyhow!("expected signature, got {}", env_sig.typ));
    }
    let sig_msg: SignatureMsg = serde_json::from_value(env_sig.payload)?;
    let sig_bytes = hex::decode(sig_msg.signature.trim())?;
    let signature = Signature::from_slice(&sig_bytes).map_err(|e| anyhow!("signature from_slice: {}", e))?;

    // verify pubkey
    let pk_bytes = hex::decode(hello.public_key.trim())?;
    let pk_arr: [u8; 32] = pk_bytes.try_into().map_err(|_| anyhow!("invalid pubkey length"))?;
    let pk = PublicKey::from_bytes(&pk_arr).map_err(|e| anyhow!("pubkey from_bytes: {}", e))?;
    pk.verify(nonce_hex.as_bytes(), &signature)
        .map_err(|e| anyhow!("signature verification failed: {}", e))?;

    let peer = PeerInfo {
        node_id: hello.node_id.clone(),
        public_key_hex: hello.public_key.clone(),
    };

    // send PeerList (from env)
    let peers_env = std::env::var("PEER_ADDRS").unwrap_or_default();
    let peers: Vec<String> = peers_env
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let pl = PeerList { peers };
    let env_pl = Envelope::new("peer_list", &pl)?;
    let bytes = serde_json::to_vec(&env_pl)?;
    framed.send(bytes.into()).await?;

    Ok((peer, framed))
}

/// Client-side handshake: perform hello->challenge->signature exchange over framed transport.
/// Returns (framed, discovered_peers).
pub async fn client_handshake_over_framed(
    mut framed: Framed<TcpStream, LengthDelimitedCodec>,
    node_id: &str,
    keypair_opt: Option<SigningKey>,
) -> Result<(Framed<TcpStream, LengthDelimitedCodec>, Vec<String>)> {
    // pick keypair
    let kp = match keypair_opt {
        Some(k) => k,
        None => {
            let k = generate_ephemeral_keypair();
            tracing::warn!("client_handshake: using ephemeral keypair pub={}", hex::encode(k.verifying_key().to_bytes()));
            k
        }
    };

    // send Hello
    let hello = Hello { node_id: node_id.to_string(), public_key: hex::encode(kp.verifying_key().to_bytes()) };
    let env_h = Envelope::new("hello", &hello)?;
    let bytes = serde_json::to_vec(&env_h)?;
    framed.send(bytes.into()).await?;

    // read challenge
    let frame = framed.next().await.ok_or_else(|| anyhow!("peer closed"))??;
    let env: Envelope = serde_json::from_slice(&frame)?;
    if env.typ != "challenge" {
        return Err(anyhow!("expected challenge, got {}", env.typ));
    }
    let challenge: Challenge = serde_json::from_value(env.payload)?;

    // sign
    let sig = kp.sign(challenge.nonce.as_bytes());
    let sig_hex = hex::encode(sig.to_bytes());
    let sig_msg = SignatureMsg { signature: sig_hex };
    let env_sig = Envelope::new("signature", &sig_msg)?;
    let bytes = serde_json::to_vec(&env_sig)?;
    framed.send(bytes.into()).await?;

    // read optional PeerList
    let mut discovered = Vec::new();
    if let Some(frame) = framed.next().await {
        let frame = frame?;
        if !frame.is_empty() {
            if let Ok(env_pl) = serde_json::from_slice::<Envelope>(&frame) {
                if env_pl.typ == "peer_list" {
                    if let Ok(pl) = serde_json::from_value::<PeerList>(env_pl.payload) {
                        discovered = pl.peers;
                    }
                }
            }
        }
    }

    Ok((framed, discovered))
}
