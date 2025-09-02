// src/network.rs
use tokio::net::{TcpListener};
use tokio::sync::mpsc;
use serde_json;
use uuid::Uuid;

use crate::dag::transaction::Transaction;

pub type TxBroadcast = mpsc::Sender<Transaction>;
pub type TxInboundReceiver = mpsc::Receiver<Transaction>;

mod handshake;
use handshake::{handle_incoming_connection, client_send_message, load_keypair_from_env};

/// Start network subsystem.
///
/// Returns (broadcast_sender, inbound_receiver).
/// - broadcast_sender: send Transaction to broadcast to peers
/// - inbound_receiver: receive Transactions from peers (already parsed JSON)
pub async fn start_network(listen_addr: &str) -> (TxBroadcast, TxInboundReceiver) {
    // broadcaster channel (other tasks will send here to broadcast to peers)
    let (bcast_tx, mut bcast_rx) = mpsc::channel::<Transaction>(256);
    // inbound channel used by listener to pass parsed txns to main
    let (inbound_tx, inbound_rx) = mpsc::channel::<Transaction>(256);

    // Node ID (stable per node run). Use NODE_ID env if present, otherwise generate one.
    let my_node_id = std::env::var("NODE_ID").unwrap_or_else(|_| Uuid::new_v4().to_string());

    // Listener: accept incoming peer connections and perform handshake + spawn read loop inside handshake handler.
    let listen_addr_str = listen_addr.to_string();
    let inbound_clone = inbound_tx.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind(&listen_addr_str)
            .await
            .expect("Failed to bind P2P listener");
        println!("P2P listener bound to {}", listen_addr_str);
        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let inbound_for_conn = inbound_clone.clone();
                    // handle handshake and spawn read-loop (inside handshake)
                    tokio::spawn(async move {
                        match handle_incoming_connection(stream, inbound_for_conn).await {
                            Ok(peer) => {
                                println!("P2P: handshake success from peer {}", peer.node_id);
                            }
                            Err(e) => {
                                println!("P2P: handshake failed from {}: {}", peer_addr, e);
                            }
                        }
                    });
                }
                Err(e) => {
                    println!("Accept error: {}", e);
                }
            }
        }
    });

    // Broadcaster task: reads from bcast_rx and sends to configured peer addrs.
    tokio::spawn(async move {
        // peers configured via env PEER_ADDRS (comma separated e.g. node2:9002,node3:9003)
        loop {
            // Gather peers each loop so env changes (or bootstrap) can be picked up dynamically.
            let peers_env = std::env::var("PEER_ADDRS").unwrap_or_default();
            let peers: Vec<String> = peers_env
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();

            // wait for next txn to broadcast
            let txn = match bcast_rx.recv().await {
                Some(t) => t,
                None => {
                    // channel closed
                    break;
                }
            };

            let serialized = match serde_json::to_string(&txn) {
                Ok(s) => s,
                Err(e) => {
                    println!("P2P: serialization error: {}", e);
                    continue;
                }
            };

            // broadcast to each peer concurrently
            for peer in peers {
                let payload = serialized.clone();
                let my_id = my_node_id.clone();
                // reload keypair from env for each connection attempt (cheap)
                tokio::spawn(async move {
                    let kp = load_keypair_from_env();
                    if let Err(e) = client_send_message(&peer, &payload, &my_id, kp).await {
                        println!("P2P: send to {} failed: {}", peer, e);
                    }
                });
            }
        }
    });

    (bcast_tx, inbound_rx)
}
