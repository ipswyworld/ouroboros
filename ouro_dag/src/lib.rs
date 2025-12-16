pub mod microchain;
pub mod escrow;
pub mod alerts;
pub mod api;
pub mod batch_writer;
pub mod bft;
pub mod config;
pub mod crypto;
pub mod dag;
pub mod keys;
pub mod mempool;
pub mod merkle;
pub mod network;
pub mod node_metrics; // Node tracking and rewards system
pub mod reconciliation;
pub mod storage;      // New storage abstraction layer
pub mod sled_storage; // Old sled-based helpers (legacy)
pub mod vm;
pub mod mainchain;
pub mod anchor_service;
pub mod subchain;
pub mod controller;
pub mod ouro_coin;
pub mod token_bucket;
pub mod tor;
pub mod multisig;
pub mod peer_discovery;
pub mod validator_registration;

use crate::reconciliation::finalize_block;

use crate::crypto::verify_ed25519_hex;

use bft::consensus::{BFTNode, HotStuff, HotStuffConfig};
use bft::state::BFTState;
use bft::validator_registry::ValidatorRegistry;
use network::bft_msg::{start_bft_server, BroadcastHandle};
use chrono::Utc;
use clap::{Parser, Subcommand};
use dag::dag::DAG;
use dag::transaction::Transaction;
use dotenvy;
use hex;
use mempool::Mempool;
use network::{start_network, TxBroadcast};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sled_storage::{batch_put, open_db, put, RocksDb};
use uuid::Uuid;
use axum::{Router, routing::get};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::BufReader;
use tokio_rustls::rustls;
use tokio_rustls::rustls::{Certificate, PrivateKey};

#[derive(Deserialize)]
pub struct IncomingFileTxn {
    sender: String,
    recipient: String,
    amount: u64,
    public_key: String,
    signature: String,
}

/// Lightweight verification stub kept for optional fallback (length checks).
/// (Kept for dev debugging but not used in production flows)
pub fn verify_signature_stub(pubkey_hex: &str, sig_hex: &str, _message: &[u8]) -> bool {
    let pk = match hex::decode(pubkey_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let sig = match hex::decode(sig_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    pk.len() == 32 && sig.len() == 64
}

/// Handle file-based transaction submission (dag_txn.json)
pub async fn handle_incoming_file(
    path: &Path,
    _dag: &mut DAG,
    mempool: &Arc<Mempool>,
    bcast: &TxBroadcast,
) {
    if !path.exists() {
        return;
    }
    let data = match tokio::fs::read_to_string(path).await {
        Ok(d) => d,
        Err(e) => {
            println!("read file error: {}", e);
            return;
        }
    };
    let parsed: IncomingFileTxn = match serde_json::from_str(&data) {
        Ok(p) => p,
        Err(e) => {
            println!("parse file txn error: {}", e);
            return;
        }
    };

    let message = format!("{}:{}:{}", parsed.sender, parsed.recipient, parsed.amount);

    // Strict verification — require real ed25519 verification (no fallback)
    let verified = verify_ed25519_hex(&parsed.public_key, &parsed.signature, message.as_bytes());
    if !verified {
        println!("❌ Signature validation failed. Transaction rejected.");
        return;
    }

    let txn = Transaction {
        id: Uuid::new_v4(),
        sender: parsed.sender.clone(),
        recipient: parsed.recipient.clone(),
        amount: parsed.amount,
        timestamp: Utc::now(),
        parents: vec![],
        signature: parsed.signature.clone(),
        public_key: parsed.public_key.clone(),
        fee: 0, // Default fee (can be extended to read from file)
        payload: None,
        chain_id: "ouroboros-mainnet-1".to_string(), // Phase 6: replay protection
        nonce: 0, // Phase 6: transaction ordering (should be queried from sender's last nonce)
    };

    if let Err(e) = mempool.add_tx(&txn) {
        println!("mempool add err: {}", e);
    } else {
        // broadcast and remove file
        let _ = bcast.send(txn.clone()).await;
        let _ = fs::remove_file(path);
        println!("✅ Verified & added transaction.");
    }
}
/// SQL splitting utility: roughly splits a SQL script into statements...
pub fn split_sql_statements(sql: &str) -> Vec<String> {
    if sql.contains("DO $") || sql.contains("DO $") {
        return vec![sql.to_string()];
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = sql.chars().peekable();

    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut dollar_tag: Option<String> = None;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            cur.push(ch);
            continue;
        }

        if in_block_comment {
            if ch == '*'
            {
                if let Some(&'/') = chars.peek() {
                    chars.next();
                    in_block_comment = false;
                    cur.push('*');
                    cur.push('/');
                    continue;
                }
            }
            cur.push(ch);
            continue;
        }

        if dollar_tag.is_some() {
            cur.push(ch);
            if ch == '$' {
                let tag = dollar_tag.as_ref().unwrap().clone();
                let mut matched = true;
                let mut tmp = String::new();
                let mut p = chars.clone();
                for tc in tag.chars().skip(1) {
                    if let Some(nc) = p.next() {
                        if nc != tc {
                            matched = false;
                            break;
                        } else {
                            tmp.push(nc);
                        }
                    } else {
                        matched = false;
                        break;
                    }
                }
                if matched {
                    for _ in 0..tmp.len() {
                        if let Some(nc) = chars.next() {
                            cur.push(nc);
                        }
                    }
                    dollar_tag = None;
                }
            }
            continue;
        }

        if ch == '-' {
            if let Some(&'-') = chars.peek() {
                chars.next();
                in_line_comment = true;
                cur.push('-');
                cur.push('-');
                continue;
            }
        }
        if ch == '/' {
            if let Some(&'*') = chars.peek() {
                chars.next();
                in_block_comment = true;
                cur.push('/');
                cur.push('*');
                continue;
            }
        }

        if ch == '\'' {
            in_single = !in_single;
            cur.push(ch);
            continue;
        }
        if ch == '"' {
            in_double = !in_double;
            cur.push(ch);
            continue;
        }

        if ch == '$' && !in_single && !in_double {
            let mut tag = String::from("$");
            let mut p = chars.clone();
            while let Some(&nc) = p.peek() {
                if nc == '$' {
                    tag.push(nc);
                    break;
                }
                if nc.is_ascii_alphanumeric() || nc == '_' {
                    tag.push(nc);
                    p.next();
                    continue;
                } else {
                    break;
                }
            }
            if tag.len() >= 2 && tag.ends_with('$') {
                for _ in 0..(tag.len() - 1) {
                    if let Some(nc) = chars.next() {
                        cur.push(nc);
                    }
                }
                dollar_tag = Some(tag);
                continue;
            } else {
                cur.push(ch);
                continue;
            }
        }

        if in_single || in_double {
            cur.push(ch);
            continue;
        }

        if ch == ';' {
            let s = cur.trim();
            if !s.is_empty() {
                out.push(s.to_string());
            }
            cur.clear();
            continue;
        }

        cur.push(ch);
    }

    let last = cur.trim();
    if !last.is_empty() {
        out.push(last.to_string());
    }
    out
}

/// Load TLS configuration from environment variables.
/// Returns None if TLS is not configured (allows fallback to HTTP).
///
/// Environment variables:
/// - `TLS_CERT_PATH`: Path to TLS certificate file (PEM format)
/// - `TLS_KEY_PATH`: Path to TLS private key file (PKCS8 PEM format)
pub fn load_tls_config() -> Option<axum_server::tls_rustls::RustlsConfig> {
    let cert_path = match std::env::var("TLS_CERT_PATH") {
        Ok(path) if !path.is_empty() => path,
        _ => {
            println!("ℹ️  TLS_CERT_PATH not set - running without TLS (HTTP only)");
            return None;
        }
    };

    let key_path = match std::env::var("TLS_KEY_PATH") {
        Ok(path) if !path.is_empty() => path,
        _ => {
            println!("⚠️  TLS_CERT_PATH set but TLS_KEY_PATH missing - running without TLS");
            return None;
        }
    };

    // Load certificate
    let cert_file = match std::fs::File::open(&cert_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("⚠️  Failed to open TLS cert '{}': {} - running without TLS", cert_path, e);
            return None;
        }
    };
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain = match certs(&mut cert_reader) {
        Ok(certs) => certs.into_iter().map(|c| Certificate(c)).collect(),
        Err(e) => {
            eprintln!("⚠️  Failed to parse TLS cert '{}': {} - running without TLS", cert_path, e);
            return None;
        }
    };

    // Load private key
    let key_file = match std::fs::File::open(&key_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("⚠️  Failed to open TLS key '{}': {} - running without TLS", key_path, e);
            return None;
        }
    };
    let mut key_reader = BufReader::new(key_file);
    let mut keys = match pkcs8_private_keys(&mut key_reader) {
        Ok(keys) => keys,
        Err(e) => {
            eprintln!("⚠️  Failed to parse TLS key '{}': {} - running without TLS", key_path, e);
            return None;
        }
    };

    if keys.is_empty() {
        eprintln!("⚠️  No private keys found in '{}' - running without TLS", key_path);
        return None;
    }

    let key = keys.remove(0);

    // Build rustls config
    let mut server_config = match rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKey(key))
    {
        Ok(config) => config,
        Err(e) => {
            eprintln!("⚠️  Failed to build TLS config: {} - running without TLS", e);
            return None;
        }
    };

    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let rustls_config = axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(server_config));

    println!("🔒 TLS enabled: cert='{}', key='{}'", cert_path, key_path);
    Some(rustls_config)
}

/// Load TLS configuration for P2P network connections.
/// Returns None if TLS is not configured (allows fallback to plain TCP).
///
/// Environment variables:
/// - `P2P_TLS_CERT_PATH`: Path to P2P TLS certificate file (PEM format)
/// - `P2P_TLS_KEY_PATH`: Path to P2P TLS private key file (PKCS8 PEM format)
///
/// If not set, falls back to using the same certs as the API (TLS_CERT_PATH/TLS_KEY_PATH)
pub fn load_p2p_tls_config() -> Option<Arc<rustls::ServerConfig>> {
    // Try P2P-specific certs first, then fall back to API certs
    let cert_path = std::env::var("P2P_TLS_CERT_PATH")
        .or_else(|_| std::env::var("TLS_CERT_PATH"))
        .ok()
        .filter(|p| !p.is_empty());

    let key_path = std::env::var("P2P_TLS_KEY_PATH")
        .or_else(|_| std::env::var("TLS_KEY_PATH"))
        .ok()
        .filter(|p| !p.is_empty());

    let (cert_path, key_path) = match (cert_path, key_path) {
        (Some(c), Some(k)) => (c, k),
        _ => {
            println!("ℹ️  P2P TLS not configured - using plain TCP for peer connections");
            return None;
        }
    };

    // Load certificate
    let cert_file = match std::fs::File::open(&cert_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("⚠️  Failed to open P2P TLS cert '{}': {} - using plain TCP", cert_path, e);
            return None;
        }
    };
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain = match certs(&mut cert_reader) {
        Ok(certs) => certs.into_iter().map(|c| Certificate(c)).collect(),
        Err(e) => {
            eprintln!("⚠️  Failed to parse P2P TLS cert '{}': {} - using plain TCP", cert_path, e);
            return None;
        }
    };

    // Load private key
    let key_file = match std::fs::File::open(&key_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("⚠️  Failed to open P2P TLS key '{}': {} - using plain TCP", key_path, e);
            return None;
        }
    };
    let mut key_reader = BufReader::new(key_file);
    let mut keys = match pkcs8_private_keys(&mut key_reader) {
        Ok(keys) => keys,
        Err(e) => {
            eprintln!("⚠️  Failed to parse P2P TLS key '{}': {} - using plain TCP", key_path, e);
            return None;
        }
    };

    if keys.is_empty() {
        eprintln!("⚠️  No private keys found in '{}' - using plain TCP", key_path);
        return None;
    }

    let key = keys.remove(0);

    // Build rustls config for P2P (no client auth required by default)
    let server_config = match rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, PrivateKey(key))
    {
        Ok(config) => config,
        Err(e) => {
            eprintln!("⚠️  Failed to build P2P TLS config: {} - using plain TCP", e);
            return None;
        }
    };

    println!("🔒 P2P TLS enabled: cert='{}', key='{}'", cert_path, key_path);
    Some(Arc::new(server_config))
}

/// Run all migrations in ./migrations in lexicographical order, while recording applied files
/// in a `schema_migrations` table. Skips statements that fail with "already exists".
pub async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    // ensure schema_migrations exists
    sqlx::query(
        r#" 
        CREATE TABLE IF NOT EXISTS schema_migrations (
            filename TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ DEFAULT now()
        );
        "#,
    )
    .execute(pool)
    .await?;

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let migrations_path = std::path::Path::new(manifest_dir).join("migrations");
    let mut read_dir = tokio::fs::read_dir(migrations_path).await?;
    let mut entries = Vec::new();
    while let Some(entry) = read_dir.next_entry().await? {
        if entry.file_name().to_string_lossy().ends_with(".sql") {
            entries.push(entry);
        }
    }

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let fname = entry.file_name().to_string_lossy().into_owned();

        // skip if already applied
        let already: Option<(String,)> =
            sqlx::query_as("SELECT filename FROM schema_migrations WHERE filename = $1")
                .bind(&fname)
                .fetch_optional(pool)
                .await?;

        if already.is_some() {
            println!("Skipping already-applied migration: {}", fname);
            continue;
        }

        let sql = tokio::fs::read_to_string(entry.path()).await?;
        println!("Running migration: {}", fname);

        let stmts = split_sql_statements(&sql);
        for (i, stmt) in stmts.into_iter().enumerate() {
            let small = if stmt.len() > 160 {
                format!("{}\
...", &stmt[..160])
            } else {
                stmt.clone()
            };
            println!(" > statement #{}: {}", i + 1, small);

            match sqlx::query(&stmt).execute(pool).await {
                Ok(_) => {} // No-op
                Err(err) => {
                    match &err {
                        sqlx::Error::Database(db_err) => {
                            let code_opt = db_err.code();
                            let msg = db_err.message().to_lowercase();
                            if code_opt == Some("42P07".into()) || msg.contains("already exists") {
                                println!("   (warning) object already exists; skipping statement");
                                continue;
                            }
                        }
                        _ => {} // No-op
                    }
                    return Err(Box::new(err));
                }
            }
        }

        // record applied filename
        sqlx::query("INSERT INTO schema_migrations (filename) VALUES ($1)")
            .bind(&fname)
            .execute(pool)
            .await?;
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Starts the ouroboros node
    Start {},
    /// Joins an existing network
    Join {
        #[arg(long)]
        peer: Option<String>,
        #[arg(long)]
        bootstrap_url: Option<String>,
        #[arg(long, default_value_t = 8000)]
        api_port: u16,
        #[arg(long, default_value_t = 9000)]
        p2p_port: u16,
        #[arg(long, default_value = "rocksdb")]
        storage: String,
        #[arg(long)]
        rocksdb_path: Option<String>,
    },
}

pub async fn run() -> std::io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start {} | Commands::Join { .. } => {
            // load .env for local development (if present)
            dotenvy::dotenv().ok();

            // Validate configuration before starting
            let config_validation = crate::config::validate_config();
            config_validation.print_summary();

            // Fail startup if configuration is invalid
            if !config_validation.valid {
                eprintln!("\n❌ Configuration validation failed! Cannot start node.");
                eprintln!("   Fix the errors above and try again.\n");
                std::process::exit(1);
            }

            // open default (sled) DB (or RocksDB if configured)
            let db_path = std::env::var("ROCKSDB_PATH").unwrap_or_else(|_| "sled_data".into());
            let db: RocksDb = open_db(&db_path);

            // Check if we're running in lightweight mode (RocksDB-only, no PostgreSQL)
            let storage_mode = std::env::var("STORAGE_MODE")
                .unwrap_or_else(|_| "postgres".into())
                .to_lowercase();

            if storage_mode.contains("rocks") {
                println!("🌐 Starting lightweight node (RocksDB-only mode, no PostgreSQL required)");
                println!("✅ RocksDB opened at: {}", db_path);

                // Start P2P network
                let listen = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:9000".into());
                let tor_config = tor::TorConfig::from_env();
                let (bcast_sender, mut inbound_rx, peer_store) = start_network(&listen, Some(tor_config)).await;
                println!("✅ P2P network started on {}", listen);

                // Debug: Check PEER_ADDRS and peer_store
                {
                    let peer_addrs_env = std::env::var("PEER_ADDRS").unwrap_or_default();
                    println!("🔍 PEER_ADDRS env var: '{}'", peer_addrs_env);

                    let store = peer_store.lock().await;
                    println!("🔍 Peer store has {} peer(s):", store.len());
                    for (i, p) in store.iter().enumerate() {
                        println!("   [{}] {} (failures: {}, last_seen: {:?})",
                            i, p.addr, p.failures, p.last_seen_unix);
                    }
                }

                // Spawn task to process inbound transactions (keep connections alive)
                tokio::spawn(async move {
                    println!("📨 Started inbound transaction processor for lightweight node");
                    while let Some(tx) = inbound_rx.recv().await {
                        // For lightweight nodes, just log received transactions
                        // (full nodes would process them into the mempool and database)
                        println!("📥 Received transaction: {} from peer", tx.id);
                    }
                    println!("📭 Inbound transaction processor stopped");
                });

                // Start minimal API server (just health check)
                let api_addr = std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".into());
                let api_addr_parsed: SocketAddr = api_addr.parse()
                    .map_err(|e| {
                        eprintln!("❌ Invalid API_ADDR format: '{}': {}", api_addr, e);
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
                    })?;

                // Create minimal router with just health endpoint
                let app = Router::new()
                    .route("/health", get(|| async { "OK" }))
                    .route("/", get(|| async { "Ouroboros Lightweight Node" }));

                println!("\n🎉 Lightweight node running!");
                println!("   P2P: {}", listen);
                println!("   API: http://{}", api_addr);
                println!("   Storage: RocksDB ({})", db_path);

                // Run API server
                println!("🚀 Starting API server (HTTP only) on http://{}", api_addr_parsed);
                if let Err(e) = axum_server::bind(api_addr_parsed)
                    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                    .await
                {
                    eprintln!(
                        "❌ API server (HTTP) crashed unexpectedly on {}: {}\
                        \n   Check if port is already in use or permissions are correct.",
                        api_addr_parsed, e
                    );
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                }
                return Ok(());
            }

            // Postgres pool for API
            let database_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/postgres".into());

            // TPS Optimization: Increase connection pool for high throughput
            let max_connections = std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(100); // Increased from 5 to 100 for 20k-50k TPS target

            println!("🔧 Database connection pool: {} max connections", max_connections);

            let db_pool = PgPoolOptions::new()
                .max_connections(max_connections)
                .acquire_timeout(Duration::from_secs(10))
                .idle_timeout(Duration::from_secs(300))
                .max_lifetime(Duration::from_secs(1800))
                .connect(&database_url)
                .await
                .map_err(|e| {
                    eprintln!(
                        "❌ Failed to connect to PostgreSQL at '{}': {}\
                        \n   Ensure PostgreSQL is running and DATABASE_URL is correct. \
                        \n   Default: postgres://postgres:postgres@127.0.0.1:5432/postgres",
                        database_url, e
                    );
                    std::io::Error::new(std::io::ErrorKind::Other, e)
                })?;

            // run migrations (fail startup if migrations fail)
            run_migrations(&db_pool)
                .await
                .map_err(|e| {
                    let msg = format!("Failed to run database migrations: {}", e);
                    eprintln!("❌ {}", msg);
                    eprintln!("   Check migrations/ directory and database permissions.");
                    std::io::Error::new(std::io::ErrorKind::Other, msg)
                })?;

            // start P2P network first so we have peer_store to pass into API router
            let listen = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:9000".into());

            // Initialize TOR configuration for hybrid clearnet + darkweb support
            let tor_config = tor::TorConfig::from_env();
            let (bcast_sender, mut inbound_rx, peer_store) = start_network(&listen, Some(tor_config)).await;

            // start API server (axum) - pass api_peer_store into router
            let api_addr = std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".into());
            let api_addr_parsed: SocketAddr = api_addr.parse()
                .map_err(|e| {
                    eprintln!(
                        "❌ Invalid API_ADDR format: '{}': {}\
                        \n   Expected format: IP:PORT (e.g., 0.0.0.0:8000)",
                        api_addr, e
                    );
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
                })?;

            // TPS Optimization: Initialize batch transaction writer
            let batch_writer = Arc::new(crate::batch_writer::BatchWriter::new(
                Arc::new(db_pool.clone()),
                db.clone(),
            ));
            println!("🚀 Batch transaction writer initialized (target: 20k-50k TPS)");

            // Build main API router
            let main_router = crate::api::router(db_pool.clone(), peer_store.clone(), batch_writer.clone());

            // Initialize additional services for subchain/microchain/mainchain APIs

            // Phase 5: Initialize Multi-Sig Coordinator for decentralized anchor posting
            let multisig_enabled = std::env::var("ENABLE_MULTISIG")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false);

            let anchor_service = if multisig_enabled {
                println!("🔐 Multi-sig anchor posting ENABLED");

                // Load validator public keys from database
                let validator_keys = match crate::multisig::MultiSigCoordinator::load_validator_keys(&db_pool).await {
                    Ok(keys) => keys,
                    Err(e) => {
                        eprintln!("⚠️  Failed to load multi-sig validator keys: {}", e);
                        eprintln!("   Falling back to single-sig mode");
                        std::collections::HashMap::new()
                    }
                };

                if !validator_keys.is_empty() {
                    let threshold = std::env::var("MULTISIG_THRESHOLD")
                        .ok()
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or_else(|| (validator_keys.len() * 2 / 3) + 1); // Default: 2/3 + 1

                    match crate::multisig::MultiSigConfig::new(threshold, validator_keys) {
                        Ok(config) => {
                            let coordinator = crate::multisig::MultiSigCoordinator::new(config);
                            println!("✅ Multi-sig coordinator initialized: {}/{} threshold", threshold, coordinator.config.total_validators);
                            Arc::new(crate::anchor_service::AnchorService::new_with_multisig(db_pool.clone(), coordinator))
                        }
                        Err(e) => {
                            eprintln!("⚠️  Multi-sig config error: {}", e);
                            eprintln!("   Falling back to single-sig mode");
                            Arc::new(crate::anchor_service::AnchorService::new(db_pool.clone()))
                        }
                    }
                } else {
                    println!("⚠️  No validator keys found, using single-sig mode");
                    Arc::new(crate::anchor_service::AnchorService::new(db_pool.clone()))
                }
            } else {
                println!("🔓 Multi-sig DISABLED (using single-sig anchor posting)");
                Arc::new(crate::anchor_service::AnchorService::new(db_pool.clone()))
            };

            // Phase 5: Initialize Validator Registry
            let validator_registry = Arc::new(crate::validator_registration::ValidatorRegistry::new(db_pool.clone()));
            println!("📋 Validator registry initialized");

            // Build sub-routers (now with authentication!)
            let subchain_router = crate::subchain::api::router(Arc::new(db_pool.clone()));
            let microchain_router = crate::microchain::api::router(Arc::new(db_pool.clone()));
            let mainchain_router = crate::mainchain::api::router(anchor_service.clone());

            // Build Ouro Coin and Token Bucket routers
            let ouro_coin_router = crate::ouro_coin::api::router(Arc::new(db_pool.clone()));
            let token_bucket_router = crate::token_bucket::api::router(Arc::new(db_pool.clone()));

            // Phase 5: Validator registration router
            let validator_router = crate::validator_registration::api::router(validator_registry.clone());

            // Combine all routers
            let router = main_router
                .nest("/subchain", subchain_router)
                .nest("/microchain", microchain_router)
                .nest("/mainchain", mainchain_router)
                .nest("/ouro", ouro_coin_router)
                .nest("/bucket", token_bucket_router)
                .nest("/validators", validator_router);

            // Load TLS configuration (optional)
            let tls_config = load_tls_config();

            // SECURITY: Enforce TLS in production mode
            let is_production = std::env::var("ENVIRONMENT")
                .map(|e| e.to_lowercase() == "production" || e.to_lowercase() == "prod")
                .unwrap_or(false);

            if is_production && tls_config.is_none() {
                eprintln!("\n🚨 CRITICAL: Production deployment REQUIRES TLS/HTTPS!");
                eprintln!("   Set TLS_CERT_PATH and TLS_KEY_PATH environment variables.");
                eprintln!("   Or set ENVIRONMENT to 'development' if this is a dev/test instance.\n");
                std::process::exit(1);
            }

            tokio::spawn(async move {
                if let Some(tls) = tls_config {
                    // HTTPS mode
                    println!("🚀 Starting API server with TLS on https://{}", api_addr_parsed);
                    if let Err(e) = axum_server::bind_rustls(api_addr_parsed, tls)
                        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                        .await
                    {
                        eprintln!(
                            "❌ API server (HTTPS) crashed unexpectedly on {}: {}\
                            \n   Check if port is already in use or permissions are correct.",
                            api_addr_parsed, e
                        );
                        std::process::exit(1);
                    }
                } else {
                    // HTTP mode (fallback)
                    println!("🚀 Starting API server (HTTP only) on http://{}", api_addr_parsed);
                    if let Err(e) = axum_server::bind(api_addr_parsed)
                        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                        .await
                    {
                        eprintln!(
                            "❌ API server (HTTP) crashed unexpectedly on {}: {}\
                            \n   Check if port is already in use or permissions are correct.",
                            api_addr_parsed, e
                        );
                        std::process::exit(1);
                    }
                }
            });

            // Initialize global storage (used by reconciliation and VM)
            sled_storage::init_global_storage(db.clone());

            // DAG
            let mut dag = DAG::new(db.clone());

            // Initialize global mempool (used by consensus via select_transactions())
            mempool::init_global_mempool(db.clone());

            // Also keep local mempool handle for API/main loop
            let mempool = Mempool::new(db.clone());
            let mempool_arc = Arc::new(mempool);

            let _validators = vec![
                BFTNode {
                    name: "NodeA".into(),
                    private_key_seed: vec![],
                    dilithium_keypair: None, // Phase 6: PQ not enabled by default
                    pq_migration_phase: crate::crypto::hybrid::MigrationPhase::Phase1EdOrHybrid,
                },
                BFTNode {
                    name: "NodeB".into(),
                    private_key_seed: vec![],
                    dilithium_keypair: None,
                    pq_migration_phase: crate::crypto::hybrid::MigrationPhase::Phase1EdOrHybrid,
                },
                BFTNode {
                    name: "NodeC".into(),
                    private_key_seed: vec![],
                    dilithium_keypair: None,
                    pq_migration_phase: crate::crypto::hybrid::MigrationPhase::Phase1EdOrHybrid,
                },
            ];

            // Initialize HotStuff BFT consensus
            let node_id = std::env::var("NODE_ID").unwrap_or_else(|_| "node-1".into());
            let bft_peers: Vec<SocketAddr> = std::env::var("BFT_PEERS")
                .unwrap_or_else(|_| "".into())
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            // Convert SocketAddr to NodeId (String) for HotStuffConfig
            let peer_node_ids: Vec<String> = bft_peers.iter()
                .map(|addr| addr.to_string())
                .collect();

            println!("🔧 Initializing HotStuff consensus:");
            println!("   Node ID: {}", node_id);
            println!("   BFT Peers (addresses): {:?}", bft_peers);
            println!("   BFT Peers (node IDs): {:?}", peer_node_ids);

            // Generate or load secret seed (32 bytes for Ed25519)
            let secret_seed = std::env::var("BFT_SECRET_SEED")
                .ok()
                .and_then(|s| hex::decode(s).ok())
                .unwrap_or_else(|| {
                    println!("⚠️  BFT_SECRET_SEED not set, using placeholder zeros (NOT FOR PRODUCTION)");
                    vec![0u8; 32]
                });

            let hotstuff_config = HotStuffConfig {
                id: node_id.clone(),
                peers: peer_node_ids,
                timeout_ms: 5000,
                secret_seed,
            };

            let broadcast_handle = BroadcastHandle::new(bft_peers.clone());
            let state = Arc::new(BFTState::new(db_pool.clone()));
            let validator_registry = Arc::new(ValidatorRegistry::new());

            let hotstuff = Arc::new(HotStuff::new(
                Arc::new(hotstuff_config),
                broadcast_handle,
                state.clone(),
                validator_registry.clone(),
            ));

            // Start BFT message server on port 9091
            let bft_port = std::env::var("BFT_PORT")
                .unwrap_or_else(|_| "9091".into())
                .parse::<u16>()
                .unwrap_or(9091);

            let bft_addr: SocketAddr = format!("0.0.0.0:{}", bft_port)
                .parse()
                .map_err(|e| {
                    eprintln!("❌ Invalid BFT_PORT configuration: {}", e);
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
                })?;

            let hotstuff_for_server = hotstuff.clone();
            tokio::spawn(async move {
                println!("🚀 Starting BFT server on {}", bft_addr);
                if let Err(e) = start_bft_server(bft_addr, hotstuff_for_server).await {
                    eprintln!("❌ BFT server error: {}", e);
                }
            });

            // load anchor key (optional)
            let anchor_key = keys::load_secret("ANCHOR_PRIVATE_KEY");
            if let Some(ref k) = anchor_key {
                println!("Loaded ANCHOR_PRIVATE_KEY (length {})", k.len());
            } else {
                eprintln!("WARNING: ANCHOR_PRIVATE_KEY not provided via Docker secret or env. Anchor operations will be disabled unless provided.");
            }

            // inbound p2p handler (spawn)
            let mempool_for_inbound = mempool_arc.clone();
            let _db_pool_for_inbound = db_pool.clone();
            tokio::spawn(async move {
                while let Some(txn) = inbound_rx.recv().await {
                    let message = format!("{}:{}:{}", txn.sender, txn.recipient, txn.amount);

                    // Strict verification — require real ed25519 verification (no fallback)
                    let verified =
                        verify_ed25519_hex(&txn.public_key, &txn.signature, message.as_bytes());
                    if !verified {
                        println!("P2P inbound txn signature invalid: {}", txn.id);
                        continue;
                    }

                    if let Err(e) = mempool_for_inbound.add_tx(&txn) {
                        println!("mempool add err (inbound): {}", e);
                    } else {
                        println!("P2P inbound txn added to mempool: {}", txn.id);
                    }
                }
            });

            let mut last_checkpoint = Instant::now();
            let checkpoint_interval = Duration::from_secs(5); // Consensus trigger interval

            println!("🧠 Ouroboros DAG engine running (consensus + p2p + mempool) ...");
            println!("   HotStuff consensus will propose blocks every {} seconds", checkpoint_interval.as_secs());

            loop {
                // check file-based submission
                let path = Path::new("dag_txn.json");
                handle_incoming_file(&path, &mut dag, &mempool_arc, &bcast_sender).await;

                // reconciliation
                reconciliation::reconcile_token_spends(&mut dag);

                // export state for debugging
                dag.export_state();

                // Consensus-driven block creation (HotStuff)
                if last_checkpoint.elapsed() >= checkpoint_interval {
                    // Trigger consensus view - HotStuff will propose a block if this node is the leader
                    println!("🔄 Triggering consensus view...");
                    if let Err(e) = hotstuff.start_view().await {
                        eprintln!("❌ Consensus view failed: {}", e);
                    }

                    // Also run legacy checkpoint for balance finalization
                    // TODO: This will be fully integrated into consensus finalization callback
                    let block_txns = mempool_arc.pop_for_block(100).unwrap_or_default();

                    if !block_txns.is_empty() {
                        let mut tx_ids = vec![];
                        let mut block_txns_ref: Vec<Transaction> = Vec::new();
                        for tx in block_txns.iter() {
                            match dag.add_transaction(tx.clone()) {
                                Ok(_) => {
                                    tx_ids.push(tx.id);
                                    block_txns_ref.push(tx.clone());
                                }
                                Err(e) => println!("dag.add_transaction failed: {}", e),
                            }
                        }

                        if !tx_ids.is_empty() {
                            let block_id = Uuid::new_v4(); // Generate a new block ID
                            if let Err(e) = finalize_block(block_id).await {
                                println!("❌ Failed to finalize block: {}", e);
                                // Re-add txs to mempool if block finalization failed
                                for tx in &block_txns_ref {
                                    if let Err(err) = mempool_arc.add_tx(tx) {
                                        println!("Failed to re-add tx {} to mempool: {}", tx.id, err);
                                    }
                                }
                                last_checkpoint = Instant::now();
                                continue;
                            }

                            // Create a Block struct for serialization and database insertion
                            let block = bft::consensus::Block {
                                id: block_id,
                                timestamp: Utc::now(),
                                tx_ids: tx_ids.clone(),
                                validator_signatures: vec![], // This will be filled by consensus
                            };
                            println!("✅ Block ID: {} at {}", block.id, block.timestamp);
                        
                                                    // execute contracts (VM)
                                                    match vm::execute_contracts(&db, &block_txns_ref) {
                                                        Ok(_res) => {
                                                            // Persist block and tx_index atomically in Postgres (authoritative)
                                                            // Use db_pool created earlier
                                                            let pg = db_pool.clone();
                                                            let mut tx_sql = match pg.begin().await {
                                                                Ok(t) => t,
                                                                Err(e) => {
                                                                    println!(
                                                                        "Failed to begin SQL transaction for block persist: {}",
                                                                        e
                                                                    );
                                                                    // Re-add txs to mempool
                                                                    for tx in &block_txns_ref {
                                                                        if let Err(err) = mempool_arc.add_tx(tx) {
                                                                            println!(
                                                                                "Failed to re-add tx {} to mempool: {}",
                                                                                tx.id, err
                                                                            );
                                                                        }
                                                                    }
                                                                    last_checkpoint = Instant::now();
                                                                    continue;
                                                                }
                                                            };
                        
                                                            // Serialize block to JSONB
                                                            let block_json: JsonValue = match serde_json::to_value(&block) {
                                                                Ok(v) => v,
                                                                Err(e) => {
                                                                    println!("Failed to serialize block to json: {}", e);
                                                                    let _ = tx_sql.rollback().await;
                                                                    for tx in &block_txns_ref {
                                                                        if let Err(err) = mempool_arc.add_tx(tx) {
                                                                            println!(
                                                                                "Failed to re-add tx {} to mempool: {}",
                                                                                tx.id, err
                                                                            );
                                                                        }
                                                                    }
                                                                    last_checkpoint = Instant::now();
                                                                    continue;
                                                                }
                                                            };
                        
                                                            // Insert into blocks
                                                            if let Err(e) = sqlx::query(
                                                                "INSERT INTO blocks (id, payload) VALUES ($1, $2)",
                                                            )
                                                            .bind(block.id,)
                                                            .bind(block_json)
                                                            .execute(&mut *tx_sql)
                                                            .await
                                                            {
                                                                println!("Failed to insert block into Postgres: {}", e);
                                                                let _ = tx_sql.rollback().await;
                                                                for tx in &block_txns_ref {
                                                                    if let Err(err) = mempool_arc.add_tx(tx) {
                                                                        println!(
                                                                            "Failed to re-add tx {} to mempool: {}",
                                                                            tx.id, err
                                                                        );
                                                                    }
                                                                }
                                                                last_checkpoint = Instant::now();
                                                                continue;
                                                            }
                        
                                                            // Insert tx_index entries
                                                            let mut failed = false;
                                                            for txid in block.tx_ids.iter() {
                                                                if let Err(e) = sqlx::query("INSERT INTO tx_index (tx_id, block_id) VALUES ($1, $2) ON CONFLICT (tx_id) DO NOTHING")
                                                                    .bind(txid,)
                                                                    .bind(block.id,)
                                                                    .execute(&mut *tx_sql)
                                                                    .await
                                                                {
                                                                    println!("Failed to insert tx_index for {}: {}", txid, e);
                                                                    failed = true;
                                                                    break;
                                                                }
                                                            }
                        
                                                            if failed {
                                                                let _ = tx_sql.rollback().await;
                                                                for tx in &block_txns_ref {
                                                                    if let Err(err) = mempool_arc.add_tx(tx) {
                                                                        println!(
                                                                            "Failed to re-add tx {} to mempool: {}",
                                                                            tx.id, err
                                                                        );
                                                                    }
                                                                }
                                                                last_checkpoint = Instant::now();
                                                                continue;
                                                            }
                        
                                                            if let Err(e) = tx_sql.commit().await {
                                                                println!("Failed to commit block transaction: {}", e);
                                                                for tx in &block_txns_ref {
                                                                    if let Err(err) = mempool_arc.add_tx(tx) {
                                                                        println!(
                                                                            "Failed to re-add tx {} to mempool: {}",
                                                                            tx.id, err
                                                                        );
                                                                    }
                                                                }
                                                                last_checkpoint = Instant::now();
                                                                continue;
                                                            }
                        
                                                            // Optionally persist to local KV (sled/rocksdb) as cache (non-authoritative)
                                                            let block_key = format!("block:{}", block.id);
                                                            if let Err(e) = put(&db, block_key.clone().into_bytes(), &block) {
                                                                println!(
                                                                    "Warning: Failed to persist block to local kv: {}",
                                                                    e
                                                                );
                                                            }
                        
                                                            let mut index_entries: Vec<(Vec<u8>, String)> = Vec::new();
                                                            for txid in block.tx_ids.iter() {
                                                                index_entries.push((
                                                                    format!("tx_index:{}", txid).into_bytes(),
                                                                    block.id.to_string(),
                                                                ));
                                                            }
                                                            if let Err(e) = batch_put(&db, index_entries) {
                                                                println!(
                                                                    "Warning: Failed to persist tx_index entries to local kv: {}",
                                                                    e
                                                                );
                                                            }
                        
                                                            println!(
                                                                "Persisted block {} and tx_index entries (Postgres authoritative)",
                                                                block.id
                                                            );
                                                        }
                                                        Err(e) => {
                                                            println!(
                                                                "❌ Contract execution failed for block {}: {}",
                                                                block.id, e
                                                            );
                                                            // Put txs back into mempool
                                                            for tx in &block_txns_ref {
                                                                if let Err(err) = mempool_arc.add_tx(tx) {
                                                                    println!("Failed to re-add tx {} to mempool after contract failure: {}", tx.id, err);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }                    }

                    last_checkpoint = Instant::now();
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
