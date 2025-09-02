// src/network/handshake.rs
use std::env;
use anyhow::{anyhow, Result};
use hex;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};

use uuid::Uuid;

use crate::dag::transaction::Transaction;
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Hello {
    pub node_id: String,
    pub public_key: String, // hex
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Challenge {
    pub nonce: String, // hex
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignatureMsg {
    pub signature: String, // hex
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub node_id: String,
    pub public_key_hex: String,
}

/// Load a Keypair from env "NODE_KEYPAIR_HEX" if present.
/// Expect hex(64 bytes) = 32 byte secret + 32 byte public.
/// Returns None if not present or invalid.
pub fn load_keypair_from_env() -> Option<Keypair> {
    match env::var("NODE_KEYPAIR_HEX") {
        Ok(hexstr) => {
            let b = match hex::decode(hexstr.trim()) {
                Ok(b) => b,
                Err(_) => return None,
            };
            if b.len() != 64 {
                return None;
            }
            Keypair::from_bytes(&b).ok()
        }
        Err(_) => None,
    }
}

/// Helper: generate a fresh ephemeral keypair (dev only)
pub fn generate_ephemeral_keypair() -> Keypair {
    let mut csprng = OsRng {};
    Keypair::generate(&mut csprng)
}

/// Server-side handler: performs handshake (Hello -> Challenge -> Signature verify),
/// then spawns a read-loop that parses newline-delimited Transaction JSON and forwards
/// them into `inbound_tx`. Returns PeerInfo if handshake succeeded.
pub async fn handle_incoming_connection(
    stream: TcpStream,
    inbound_tx: mpsc::Sender<Transaction>,
) -> Result<PeerInfo> {
    let peer_addr = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
    // split stream into reader/writer halves
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);

    // 1) Read Hello (one line)
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    if line.trim().is_empty() {
        return Err(anyhow!("empty hello from peer {}", peer_addr));
    }
    let hello: Hello = serde_json::from_str(line.trim_end())
        .map_err(|e| anyhow!("hello parse error: {}", e))?;

    // 2) Send challenge
    let mut nonce = [0u8; 16];
    OsRng.fill_bytes(&mut nonce);
    let nonce_hex = hex::encode(&nonce);
    let challenge = Challenge { nonce: nonce_hex.clone() };
    let challenge_json = serde_json::to_string(&challenge)?;
    writer.write_all(challenge_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // 3) Read signature msg
    line.clear();
    reader.read_line(&mut line).await?;
    let sig_msg: SignatureMsg = serde_json::from_str(line.trim_end())
        .map_err(|e| anyhow!("signature msg parse error: {}", e))?;
    let sig_bytes = hex::decode(sig_msg.signature.trim())
        .map_err(|e| anyhow!("sig hex decode: {}", e))?;
    let signature = Signature::from_bytes(&sig_bytes)
        .map_err(|e| anyhow!("sig from_bytes failed: {}", e))?;

    // 4) Verify signature using the provided public key
    let pk_bytes = hex::decode(hello.public_key.trim())
        .map_err(|e| anyhow!("pubkey hex decode: {}", e))?;
    let pk = PublicKey::from_bytes(&pk_bytes)
        .map_err(|e| anyhow!("pubkey from_bytes: {}", e))?;

    pk.verify(nonce_hex.as_bytes(), &signature)
        .map_err(|e| anyhow!("signature verification failed: {}", e))?;

    tracing::info!(%peer_addr, %hello.node_id, "peer authenticated");

    let peer = PeerInfo {
        node_id: hello.node_id.clone(),
        public_key_hex: hello.public_key.clone(),
    };

    // Spawn the per-connection read loop that reads subsequent newline-delimited messages
    // (we continue using the same reader).
    let mut reader_for_loop = reader;
    let inbound = inbound_tx.clone();
    tokio::spawn(async move {
        let mut buf = String::new();
        loop {
            buf.clear();
            match reader_for_loop.read_line(&mut buf).await {
                Ok(0) => {
                    tracing::info!(%peer_addr, "peer disconnected (EOF)");
                    break;
                }
                Ok(_) => {
                    let s = buf.trim_end();
                    if s.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<Transaction>(s) {
                        Ok(txn) => {
                            if let Err(_e) = inbound.send(txn).await {
                                tracing::warn!(%peer_addr, "inbound channel closed; dropping message");
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!(%peer_addr, "failed to parse Transaction: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(%peer_addr, "connection read error: {}", e);
                    break;
                }
            }
        }
    });

    Ok(peer)
}

/// Client-side helper: connect to peer_addr, perform handshake (Hello -> wait Challenge -> sign -> send signature),
/// then send `payload` (a newline-delimited JSON string). This is suitable for short-lived broadcast connections.
pub async fn client_send_message(
    peer_addr: &str,
    payload: &str,
    node_id: &str,
    keypair_opt: Option<Keypair>,
) -> Result<()> {
    // connect
    let stream = TcpStream::connect(peer_addr).await?;
    let peer_addr_display = peer_addr.to_string();
    // split
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);

    // pick keypair (provided or ephemeral)
    let kp = match keypair_opt {
        Some(k) => k,
        None => {
            let k = generate_ephemeral_keypair();
            tracing::warn!(
                "client_send_message: no NODE_KEYPAIR_HEX; using ephemeral keypair pub={}",
                hex::encode(k.public.to_bytes())
            );
            k
        }
    };

    // send Hello
    let hello = Hello {
        node_id: node_id.to_string(),
        public_key: hex::encode(kp.public.to_bytes()),
    };
    let hello_json = serde_json::to_string(&hello)?;
    writer.write_all(hello_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // read challenge
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let challenge: Challenge = serde_json::from_str(line.trim_end())
        .map_err(|e| anyhow!("challenge parse: {}", e))?;

    // sign
    let sig = kp.sign(challenge.nonce.as_bytes());
    let sig_hex = hex::encode(sig.to_bytes());
    let sig_msg = SignatureMsg { signature: sig_hex };
    let sig_json = serde_json::to_string(&sig_msg)?;
    writer.write_all(sig_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    // now send the payload (a newline-terminated JSON string)
    writer.write_all(payload.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    tracing::info!(%peer_addr_display, "client_send_message: sent payload");
    Ok(())
}
