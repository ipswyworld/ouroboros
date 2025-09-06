// src/network.rs
pub mod handshake;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json;

use std::collections::{HashMap, HashSet};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::sleep;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;

use crate::dag::transaction::Transaction;
use self::handshake::{Envelope, message_id_from_envelope};

use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Lightweight metrics
static METRICS_ACTIVE_CONNECTIONS: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
static METRICS_DEDUPE_ENTRIES: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
static METRICS_PEER_COUNT: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

pub type TxBroadcast = mpsc::Sender<Transaction>;
pub type TxInboundReceiver = mpsc::Receiver<Transaction>;

/// Peer entry stored in runtime store and persisted to peers.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEntry {
    pub addr: String,
    pub last_seen_unix: Option<u64>,
    pub failures: u32,
    pub banned_until_unix: Option<u64>,
    // rate limit window
    pub rate_window_start_unix: Option<u64>,
    pub rate_count: u32,
}

impl PeerEntry {
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            last_seen_unix: Some(current_unix()),
            failures: 0,
            banned_until_unix: None,
            rate_window_start_unix: Some(current_unix()),
            rate_count: 0,
        }
    }
}

pub type PeerStore = Arc<Mutex<Vec<PeerEntry>>>;

/// Connection handle representing a persistent outbound connection to a peer.
pub struct Connection {
    pub addr: String,
    pub tx: mpsc::Sender<Envelope>, // send envelopes to this connection task
    pub last_seen: Arc<Mutex<Option<Instant>>>,
}

type DedupeCache = Arc<Mutex<HashMap<String, Instant>>>;

/// helper: current epoch seconds
fn current_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

async fn save_peers_to_file(store: &PeerStore) {
    let peers = store.lock().await;
    if let Ok(json) = serde_json::to_string(&*peers) {
        let _ = tokio::fs::write("peers.json", json).await;
    }
}

async fn load_peers_from_file() -> Vec<PeerEntry> {
    if let Ok(b) = tokio::fs::read_to_string("peers.json").await {
        if let Ok(v) = serde_json::from_str::<Vec<PeerEntry>>(&b) {
            return v;
        }
    }
    Vec::new()
}

async fn fetch_bootstrap_peers(url: &str) -> Result<Vec<String>, anyhow::Error> {
    let body = reqwest::get(url).await?.text().await?;
    let peers: Vec<String> = body.lines().map(|l| l.trim().to_string()).filter(|s| !s.is_empty()).collect();
    Ok(peers)
}

/// prune peer_store to MAX_PEERS and remove very stale entries
fn prune_peer_list(list: &mut Vec<PeerEntry>) {
    const MAX_PEERS: usize = 2000;
    const TTL_SECS: u64 = 60 * 60 * 24 * 7; // 7 days
    let cutoff = current_unix().saturating_sub(TTL_SECS);
    list.retain(|e| e.last_seen_unix.unwrap_or(0) >= cutoff || e.failures < 8);
    if list.len() > MAX_PEERS {
        list.sort_by_key(|e| std::cmp::Reverse(e.last_seen_unix.unwrap_or(0)));
        list.truncate(MAX_PEERS);
    }
}

/// Returns lightweight p2p metrics (used by API)
pub fn get_p2p_metrics() -> (usize, usize, usize) {
    let conns = METRICS_ACTIVE_CONNECTIONS.load(Ordering::Relaxed);
    let dedupe = METRICS_DEDUPE_ENTRIES.load(Ordering::Relaxed);
    let peers = METRICS_PEER_COUNT.load(Ordering::Relaxed);
    (conns, dedupe, peers)
}

/// Try UPnP mapping if enabled (non-blocking best-effort).
async fn try_upnp_map(listen_addr: &str) {
    if std::env::var("USE_UPNP").is_err() {
        return;
    }
    // parse listen port
    if let Some(pos) = listen_addr.rfind(':') {
        if let Ok(port) = listen_addr[pos + 1..].parse::<u16>() {
            // run mapping on background
            tokio::spawn(async move {
                match igd::aio::search_gateway(Default::default()).await {
                    Ok(gateway) => {
                        let local_port = port;
                        let external_port = std::env::var("EXTERNAL_PORT").ok().and_then(|s| s.parse::<u16>().ok()).unwrap_or(local_port);
                        let local_ip_str = local_ipaddress::get().unwrap_or_else(|| "127.0.0.1".to_string());
                        let local_ipv4: Ipv4Addr = local_ip_str.parse().unwrap_or(Ipv4Addr::new(127, 0, 0, 1));
                        let local_socket_addr = SocketAddrV4::new(local_ipv4, local_port);
                        let lifetime = 60 * 60 * 24; // 1 day
                        match gateway.add_port(igd::PortMappingProtocol::TCP, external_port, local_socket_addr, lifetime, "ouro_p2p").await {
                            Ok(_) => {
                                tracing::info!("UPnP: mapped external {} -> local {} (ttl {})", external_port, local_port, lifetime);
                            }
                            Err(e) => {
                                tracing::warn!("UPnP mapping failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("UPnP gateway search failed: {}", e);
                    }
                }
            });
        }
    }
}

/// Start network subsystem.
///
/// Returns (broadcast_sender, inbound_receiver, peer_store).
/// The function will internally spawn tasks and a Ctrl+C-based graceful shutdown
/// broadcaster that tasks listen to. Tasks are written to exit cleanly on shutdown.
pub async fn start_network(listen_addr: &str) -> (TxBroadcast, TxInboundReceiver, PeerStore) {
    let (bcast_tx, mut bcast_rx) = mpsc::channel::<Transaction>(256);
    let (inbound_tx, inbound_rx) = mpsc::channel::<Transaction>(256);

    // Prepare shutdown broadcaster (tasks subscribe to it)
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    {
        // spawn a Ctrl+C handler that broadcasts shutdown
        let tx = shutdown_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::warn!("failed to install Ctrl+C handler: {}", e);
                return;
            }
            tracing::info!("shutdown signal received, notifying tasks...");
            let _ = tx.send(());
        });
    }

    // attempt UPnP mapping if configured (best-effort)
    try_upnp_map(listen_addr).await;

    let peer_store: PeerStore = Arc::new(Mutex::new(load_peers_from_file().await));
    {
        let mut s = peer_store.lock().await;
        let peers_env = std::env::var("PEER_ADDRS").unwrap_or_default();
        for p in peers_env.split(',').map(|p| p.trim()) {
            if !p.is_empty() && !s.iter().any(|e| e.addr == p) {
                s.push(PeerEntry::new(p.to_string()));
            }
        }
        prune_peer_list(&mut s);
        METRICS_PEER_COUNT.store(s.len(), Ordering::Relaxed);
        let _ = save_peers_to_file(&peer_store).await;
    }

    // dedupe cache (msgid -> expiry Instant)
    let dedupe: DedupeCache = Arc::new(Mutex::new(HashMap::new()));
    {
        let ded = dedupe.clone();
        let shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            let mut shutdown_rx = shutdown_rx;
            loop {
                tokio::select! {
                    _ = sleep(Duration::from_secs(30)) => {
                        let mut guard = ded.lock().await;
                        let now = Instant::now();
                        guard.retain(|_, expiry| *expiry > now);
                        METRICS_DEDUPE_ENTRIES.store(guard.len(), Ordering::Relaxed);
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("dedupe prune task shutting down");
                        break;
                    }
                }
            }
        });
    }

    // connections map addr -> Connection
    let connections: Arc<Mutex<HashMap<String, Connection>>> = Arc::new(Mutex::new(HashMap::new()));

    // Node id
    let my_node_id = std::env::var("NODE_ID").unwrap_or_else(|_| Uuid::new_v4().to_string());

    // Listener - incoming connections
    let listen_addr_str = listen_addr.to_string();
    let inbound_clone = inbound_tx.clone();
    let peer_store_for_listener = peer_store.clone();
    let dedupe_for_listener = dedupe.clone();
    let mut shutdown_rx_for_listener = shutdown_tx.subscribe();
    tokio::spawn(async move {
        let listener = match TcpListener::bind(&listen_addr_str).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind P2P listener {}: {}", listen_addr_str, e);
                return;
            }
        };
        tracing::info!("P2P listener bound to {}", listen_addr_str);
        loop {
            tokio::select! {
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((stream, peer_addr)) => {
                            let inbound_for_conn = inbound_clone.clone();
                            let ps = peer_store_for_listener.clone();
                            let dedupe_clone = dedupe_for_listener.clone();
                            tokio::spawn(async move {
                                match handshake::server_handshake_and_upgrade(stream).await {
                                    Ok((peer_info, mut framed)) => {
                                        tracing::info!(peer = %peer_info.node_id, addr = %peer_addr, "handshake success");
                                        // add to peer_store using remote socket address
                                        let remote_addr = peer_addr.to_string();
                                        {
                                            let mut store = ps.lock().await;
                                            if !store.iter().any(|e| e.addr == remote_addr) {
                                                store.push(PeerEntry::new(remote_addr.clone()));
                                                prune_peer_list(&mut store);
                                                METRICS_PEER_COUNT.store(store.len(), Ordering::Relaxed);
                                                let _ = save_peers_to_file(&ps).await;
                                            } else {
                                                // update last seen
                                                if let Some(e) = store.iter_mut().find(|x| x.addr == remote_addr) {
                                                    e.last_seen_unix = Some(current_unix());
                                                    e.failures = 0;
                                                }
                                            }
                                        }

                                        // read loop: framed stream -> handle envelopes
                                        while let Some(frame) = framed.next().await {
                                            match frame {
                                                Ok(bytes) => {
                                                    if bytes.is_empty() { continue; }
                                                    // limit max envelope size (simple protection)
                                                    if bytes.len() > 64 * 1024 {
                                                        tracing::warn!("incoming envelope too large from {}: {} bytes", peer_addr, bytes.len());
                                                        continue;
                                                    }
                                                    if let Ok(env) = serde_json::from_slice::<Envelope>(bytes.as_ref()) {
                                                        // validate envelope shape minimally
                                                        if env.typ.is_empty() {
                                                            tracing::warn!("incoming envelope missing type; ignoring");
                                                            continue;
                                                        }
                                                        // dedupe inbound
                                                        let msgid = message_id_from_envelope(&env);
                                                        let mut ded = dedupe_clone.lock().await;
                                                        if let Some(exp) = ded.get(&msgid) {
                                                            if *exp > Instant::now() { continue; }
                                                        }
                                                        ded.insert(msgid, Instant::now() + Duration::from_secs(300));
                                                        drop(ded);

                                                        if env.typ == "gossip_tx" {
                                                            if let Ok(txn) = serde_json::from_value::<Transaction>(env.payload.clone()) {
                                                                let _ = inbound_for_conn.send(txn).await;
                                                            }
                                                        } else if env.typ == "peer_list" {
                                                            if let Ok(pl) = serde_json::from_value::<handshake::PeerList>(env.payload) {
                                                                let mut store = ps.lock().await;
                                                                let mut changed = false;
                                                                for p in pl.peers {
                                                                    if !store.iter().any(|e| e.addr == p) {
                                                                        store.push(PeerEntry::new(p.clone()));
                                                                        changed = true;
                                                                    }
                                                                }
                                                                if changed {
                                                                    prune_peer_list(&mut store);
                                                                    METRICS_PEER_COUNT.store(store.len(), Ordering::Relaxed);
                                                                    let _ = save_peers_to_file(&ps).await;
                                                                }
                                                            }
                                                        } else if env.typ == "ping" {
                                                            // no-op here
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::warn!("P2P inbound read error from {}: {}", peer_addr, e);
                                                    break;
                                                }
                                            }
                                        }
                                        tracing::info!("connection read loop ended for {}", peer_addr);
                                    }
                                    Err(e) => {
                                        tracing::warn!("P2P: handshake failed from {}: {}", peer_addr, e);
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!("Accept error: {}", e);
                        }
                    }
                }
                _ = shutdown_rx_for_listener.recv() => {
                    tracing::info!("listener shutting down");
                    break;
                }
            }
        }
    });

    // Connection manager: ensure outbound persistent connections
    let connections_for_manager = connections.clone();
    let peer_store_for_manager = peer_store.clone();
    let dedupe_for_manager = dedupe.clone();
    let inbound_for_manager = inbound_tx.clone();
    let shutdown_rx_for_manager = shutdown_tx.subscribe();
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        let mut shutdown_rx = shutdown_rx_for_manager;
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("connection manager shutting down");
                    break;
                }
                _ = sleep(Duration::from_secs(0)) => {
                    // iterate peers and ensure connections
                    {
                        let peers = peer_store_for_manager.lock().await.clone();
                        for p in peers.iter() {
                            if let Some(banned_until) = p.banned_until_unix {
                                if banned_until > current_unix() { continue; }
                            }
                            let addr = p.addr.clone();
                            let mut conns = connections_for_manager.lock().await;
                            if conns.contains_key(&addr) { continue; }

                            let (tx, rx) = mpsc::channel::<Envelope>(128);
                            let last_seen = Arc::new(Mutex::new(Some(Instant::now())));
                            let conn = Connection { addr: addr.clone(), tx: tx.clone(), last_seen: last_seen.clone() };
                            conns.insert(addr.clone(), conn);
                            METRICS_ACTIVE_CONNECTIONS.fetch_add(1, Ordering::Relaxed);

                            // spawn per-peer connection task
                            let peer_store_clone = peer_store_for_manager.clone();
                            let connections_clone2 = connections_for_manager.clone();
                            let dedupe_clone2 = dedupe_for_manager.clone();
                            let inbound_clone2 = inbound_for_manager.clone();
                            let my_id_clone = my_node_id.clone();
                            let mut shutdown_inner = shutdown_tx_clone.subscribe();
                            tokio::spawn(async move {
                                let connection_result = async {
                                    // check for shutdown early
                                    if shutdown_inner.try_recv().is_ok() { 
                                        return Err(anyhow::anyhow!("shutting down"));
                                    }

                                    let stream = TcpStream::connect(&addr).await.map_err(|e| anyhow::anyhow!("connect failed: {}", e))?;
                                    let codec = LengthDelimitedCodec::new();
                                    let framed = Framed::new(stream, codec);
                                    handshake::client_handshake_over_framed(framed, &my_id_clone, handshake::load_keypair_from_env()).await.map_err(|e| anyhow::anyhow!("handshake failed: {}", e))
                                }.await;

                                match connection_result {
                                    Ok((framed_conn, discovered)) => {
                                        // split sink/stream so we can have independent send and receive tasks
                                        let (mut sink, mut stream) = framed_conn.split();

                                        // merge discovered peers
                                        {
                                            let mut store = peer_store_clone.lock().await;
                                            let mut changed = false;
                                            for d in discovered {
                                                if !store.iter().any(|e| e.addr == d) {
                                                    store.push(PeerEntry::new(d.clone()));
                                                    changed = true;
                                                }
                                            }
                                            if changed {
                                                prune_peer_list(&mut store);
                                                METRICS_PEER_COUNT.store(store.len(), Ordering::Relaxed);
                                                let _ = save_peers_to_file(&peer_store_clone).await;
                                            }
                                        }

                                        // reset failure count & update last_seen
                                        {
                                            let mut store = peer_store_clone.lock().await;
                                            if let Some(e) = store.iter_mut().find(|e| e.addr == addr) {
                                                e.failures = 0;
                                                e.last_seen_unix = Some(current_unix());
                                            }
                                        }

                                        // spawn sender task: rx -> sink
                                        let mut send_rx = rx;
                                        let send_addr = addr.clone();
                                        let peer_store_clone2 = peer_store_clone.clone();
                                        let sender_handle = tokio::spawn(async move {
                                            // keepalive timer
                                            let mut keepalive = tokio::time::interval(Duration::from_secs(15));
                                            loop {
                                                tokio::select! {
                                                    biased;

                                                    maybe_env = send_rx.recv() => {
                                                        match maybe_env {
                                                            Some(env) => {
                                                                // simple per-peer rate limiting check using peer_store
                                                                let mut allow = true;
                                                                {
                                                                    let mut store = peer_store_clone2.lock().await;
                                                                    if let Some(entry) = store.iter_mut().find(|e| e.addr == send_addr) {
                                                                        let window = entry.rate_window_start_unix.unwrap_or(current_unix());
                                                                        let now = current_unix();
                                                                        const WINDOW_SECS: u64 = 60;
                                                                        const MAX_PER_WINDOW: u32 = 600; // default ceiling
                                                                        if now >= window + WINDOW_SECS {
                                                                            entry.rate_window_start_unix = Some(now);
                                                                            entry.rate_count = 0;
                                                                        }
                                                                        if entry.rate_count >= MAX_PER_WINDOW {
                                                                            allow = false;
                                                                        } else {
                                                                            entry.rate_count = entry.rate_count.saturating_add(1);
                                                                        }
                                                                    }
                                                                }
                                                                if !allow {
                                                                    tracing::warn!("rate limit reached for {}; dropping outgoing envelope", send_addr);
                                                                    continue;
                                                                }

                                                                let bytes = match serde_json::to_vec(&env) {
                                                                    Ok(b) => b,
                                                                    Err(e) => {
                                                                        tracing::warn!("serialize envelope err: {}", e);
                                                                        continue;
                                                                    }
                                                                };
                                                                if let Err(e) = sink.send(bytes.into()).await {
                                                                    tracing::warn!("send to {} failed: {}", &send_addr, e);
                                                                    break;
                                                                }
                                                            }
                                                            None => {
                                                                // channel closed; exit sender
                                                                break;
                                                            }
                                                        }
                                                    }

                                                    _ = keepalive.tick() => {
                                                        // send a ping envelope
                                                        let ping = match Envelope::new("ping", &serde_json::json!({"ts": current_unix()})) {
                                                            Ok(e) => e,
                                                            Err(_) => continue,
                                                        };
                                                        let bytes = match serde_json::to_vec(&ping) {
                                                            Ok(b) => b,
                                                            Err(_) => continue,
                                                        };
                                                        if let Err(e) = sink.send(bytes.into()).await {
                                                            tracing::warn!("keepalive send to {} failed: {}", &send_addr, e);
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        });

                                        // read loop: stream -> handle incoming envelopes
                                        while let Some(item) = stream.next().await {
                                            match item {
                                                Ok(bytes) => {
                                                    if bytes.is_empty() { continue; }
                                                    if bytes.len() > 64 * 1024 { continue; }
                                                    if let Ok(env) = serde_json::from_slice::<Envelope>(bytes.as_ref()) {
                                                        if env.typ.is_empty() { continue; }
                                                        let msgid = message_id_from_envelope(&env);
                                                        {
                                                            let mut ded = dedupe_clone2.lock().await;
                                                            if let Some(exp) = ded.get(&msgid) {
                                                                if *exp > Instant::now() { continue; }
                                                            }
                                                            ded.insert(msgid, Instant::now() + Duration::from_secs(300));
                                                        }

                                                        match env.typ.as_str() {
                                                            "gossip_tx" => {
                                                                if let Ok(txn) = serde_json::from_value::<Transaction>(env.payload.clone()) {
                                                                    let _ = inbound_clone2.send(txn).await;
                                                                }
                                                            }
                                                            "pong" => {
                                                                if let Some(conn) = connections_clone2.lock().await.get_mut(&addr) {
                                                                    let mut ls = conn.last_seen.lock().await;
                                                                    *ls = Some(Instant::now());
                                                                }
                                                                let mut store = peer_store_clone.lock().await;
                                                                if let Some(e) = store.iter_mut().find(|x| x.addr == addr) {
                                                                    e.last_seen_unix = Some(current_unix());
                                                                    e.failures = 0;
                                                                    let _ = save_peers_to_file(&peer_store_clone).await;
                                                                }
                                                            }
                                                            "peer_list" => {
                                                                if let Ok(pl) = serde_json::from_value::<handshake::PeerList>(env.payload) {
                                                                    let mut store = peer_store_clone.lock().await;
                                                                    let mut changed = false;
                                                                    for p in pl.peers {
                                                                        if !store.iter().any(|e| e.addr == p) {
                                                                            store.push(PeerEntry::new(p.clone()));
                                                                            changed = true;
                                                                        }
                                                                    }
                                                                    if changed {
                                                                        prune_peer_list(&mut store);
                                                                        METRICS_PEER_COUNT.store(store.len(), Ordering::Relaxed);
                                                                        let _ = save_peers_to_file(&peer_store_clone).await;
                                                                    }
                                                                }
                                                            }
                                                            "ping" => {
                                                                let pong = Envelope::new("pong", &serde_json::json!({"ts": current_unix()})).unwrap();
                                                                if let Err(e) = tx.try_send(pong) {
                                                                    tracing::warn!("failed to queue pong to {}: {}", &addr, e);
                                                                }
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::warn!("read error from {}: {}", &addr, e);
                                                    break;
                                                }
                                            }
                                        }

                                        // connection closed / read loop ended -> kill sender
                                        sender_handle.abort();
                                    }
                                    Err(e) => {
                                        tracing::warn!("outbound connection to {} failed: {}", &addr, e);
                                        // mark failure
                                        let mut store = peer_store_clone.lock().await;
                                        if let Some(ent) = store.iter_mut().find(|e| e.addr == addr) {
                                            ent.failures = ent.failures.saturating_add(1);
                                            if ent.failures >= 5 {
                                                ent.banned_until_unix = Some(current_unix() + 60 * 5);
                                            }
                                            let _ = save_peers_to_file(&peer_store_clone).await;
                                        }
                                    }
                                }

                                // remove connection mapping at the end of the task's life
                                connections_clone2.lock().await.remove(&addr);
                                METRICS_ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
                            });
                        }
                    }

                    // cleanup: remove connections not in peer list
                    {
                        let peers = peer_store_for_manager.lock().await;
                        let mut conns = connections_for_manager.lock().await;
                        let allowed: HashSet<String> = peers.iter().map(|p| p.addr.clone()).collect();
                        conns.retain(|k, _| allowed.contains(k));
                    }
                }
            }
        }
    });

    // Broadcaster: read bcast_rx, create Envelope, dedupe, fan out to connection txs (bounded)
    let connections_for_bcast = connections.clone();
    let dedupe_for_bcast = dedupe.clone();
    let peer_store_for_bcast = peer_store.clone();
    let shutdown_rx_for_bcast = shutdown_tx.subscribe();
    tokio::spawn(async move {
        const MAX_FANOUT: usize = 8;
        let mut shutdown_rx = shutdown_rx_for_bcast;
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("broadcaster shutting down");
                    break;
                }
                maybe_txn = bcast_rx.recv() => {
                    let txn = match maybe_txn {
                        Some(t) => t,
                        None => break,
                    };

                    // envelope creation
                    let env = match Envelope::new("gossip_tx", &txn) {
                        Ok(e) => e,
                        Err(e) => {
                            tracing::warn!("failed to create gossip envelope: {}", e);
                            continue;
                        }
                    };

                    let msgid = message_id_from_envelope(&env);

                    // dedupe outbound
                    {
                        let mut d = dedupe_for_bcast.lock().await;
                        if let Some(exp) = d.get(&msgid) {
                            if *exp > Instant::now() { continue; }
                        }
                        d.insert(msgid.clone(), Instant::now() + Duration::from_secs(300));
                        METRICS_DEDUPE_ENTRIES.store(d.len(), Ordering::Relaxed);
                    }

                    // snapshot connections
                    let conns_snapshot = {
                        let conns = connections_for_bcast.lock().await;
                        conns.iter().map(|(k, v)| (k.clone(), v.tx.clone())).collect::<Vec<_>>()
                    };

                    if conns_snapshot.is_empty() {
                        // try bootstrap if we have none
                        if let Ok(url) = std::env::var("BOOTSTRAP_URL") {
                            if let Ok(remote) = fetch_bootstrap_peers(&url).await {
                                let mut store = peer_store_for_bcast.lock().await;
                                let mut changed = false;
                                for p in remote {
                                    if !store.iter().any(|e| e.addr == p) {
                                        store.push(PeerEntry::new(p.clone()));
                                        changed = true;
                                    }
                                }
                                if changed { let _ = save_peers_to_file(&peer_store_for_bcast).await; METRICS_PEER_COUNT.store(store.len(), Ordering::Relaxed); }
                            }
                        }
                    }

                    // bounded fanout selection
                    let mut targets: Vec<(String, mpsc::Sender<Envelope>)> = Vec::new();
                    if !conns_snapshot.is_empty() {
                        let n = conns_snapshot.len();
                        let start_byte = msgid.as_bytes()[0] as usize;
                        let mut idx = start_byte % n;
                        let mut picked = 0usize;
                        while picked < std::cmp::min(MAX_FANOUT, n) {
                            let (addr, tx) = &conns_snapshot[idx];
                            targets.push((addr.clone(), tx.clone()));
                            picked += 1;
                            idx = (idx + 1) % n;
                        }
                    }

                    // fan out to selected peers concurrently (best-effort)
                    for (peer_addr, tx) in targets {
                        let env_clone = env.clone();
                        let peer_store_inner = peer_store_for_bcast.clone();
                        tokio::spawn(async move {
                            // quick rate-limit check from peer_store
                            let mut allow = true;
                            {
                                let mut store = peer_store_inner.lock().await;
                                if let Some(entry) = store.iter_mut().find(|e| e.addr == peer_addr) {
                                    let now = current_unix();
                                    const WINDOW_SECS: u64 = 60;
                                    const MAX_PER_WINDOW: u32 = 600;
                                    let start = entry.rate_window_start_unix.unwrap_or(now);
                                    if now >= start + WINDOW_SECS {
                                        entry.rate_window_start_unix = Some(now);
                                        entry.rate_count = 0;
                                    }
                                    if entry.rate_count >= MAX_PER_WINDOW {
                                        allow = false;
                                    } else {
                                        entry.rate_count = entry.rate_count.saturating_add(1);
                                    }
                                }
                            }
                            if !allow {
                                tracing::warn!("skipping send to {} due rate limit", peer_addr);
                                return;
                            }

                            if let Err(e) = tx.try_send(env_clone) {
                                tracing::warn!("P2P: failed to queue envelope to {}: {}", peer_addr, e);
                                // record failure
                                let mut store = peer_store_inner.lock().await;
                                if let Some(entry) = store.iter_mut().find(|e| e.addr == peer_addr) {
                                    entry.failures = entry.failures.saturating_add(1);
                                    if entry.failures >= 5 {
                                        entry.banned_until_unix = Some(current_unix() + 60 * 5);
                                    }
                                    let _ = save_peers_to_file(&peer_store_inner).await;
                                }
                            }
                        });
                    }

                    sleep(Duration::from_millis(5)).await;
                }
            }
        }
    });

    (bcast_tx, inbound_rx, peer_store)
}
