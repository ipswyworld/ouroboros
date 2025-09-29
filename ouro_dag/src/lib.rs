pub mod api;
pub mod bft;
pub mod crypto;
pub mod dag;
pub mod keys;
pub mod mempool;
pub mod network;
pub mod reconciliation;
pub mod storage;
pub mod vm;

use crate::crypto::verify_ed25519_hex;

use bft::consensus::{finalize_block, BFTNode};
use chrono::Utc;
use clap::Parser;
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
use storage::{batch_put, open_db, put, RocksDb};
use uuid::Uuid;

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
    let data = match fs::read_to_string(path) {
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
        payload: None,
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
    let mut entries: Vec<_> = std::fs::read_dir(migrations_path)?
        .filter_map(|r| r.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".sql"))
        .collect();

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

        let sql = std::fs::read_to_string(entry.path())?;
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

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Starts the ouroboros node
    Start {},
}

pub async fn run() -> std::io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start {} => {
            // load .env for local development (if present)
            dotenvy::dotenv().ok();

            // open default (sled) DB (or RocksDB if configured)
            let db_path = std::env::var("ROCKSDB_PATH").unwrap_or_else(|_| "sled_data".into());
            let db: RocksDb = open_db(&db_path);

            // Postgres pool for API
            let database_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/postgres".into());
            let db_pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .expect("failed to connect to postgres (set DATABASE_URL env or run a local postgres)");

            // run migrations (fail startup if migrations fail)
            

            // start P2P network first so we have peer_store to pass into API router
            let listen = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:9000".into());
            let (bcast_sender, mut inbound_rx, peer_store) = start_network(&listen).await;

            // start API server (axum) - pass api_peer_store into router
            let api_addr = std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".into());
            let api_addr_parsed: SocketAddr = api_addr.parse().expect("API_ADDR invalid");
            let router = crate::api::router(db_pool.clone(), peer_store.clone());
            tokio::spawn(async move {
                println!("Starting API server on {}", api_addr_parsed);
                axum::Server::bind(&api_addr_parsed)
                    .serve(router.into_make_service())
                    .await
                    .expect("API server crashed");
            });

            // DAG
            let mut dag = DAG::new(db.clone());

            // mempool
            let mempool = Mempool::new(db.clone());
            let mempool_arc = Arc::new(mempool);

            // validators (simulated)
            let validators = vec![
                BFTNode {
                    name: "NodeA".into(),
                    private_key: "key1".into(),
                },
                BFTNode {
                    name: "NodeB".into(),
                    private_key: "key2".into(),
                },
                BFTNode {
                    name: "NodeC".into(),
                    private_key: "key3".into(),
                },
            ];

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
            let checkpoint_interval = Duration::from_secs(30);

            println!("🧠 Ouroboros DAG engine running (p2p + mempool) ...");

            loop {
                // check file-based submission
                let path = Path::new("dag_txn.json");
                handle_incoming_file(&path, &mut dag, &mempool_arc, &bcast_sender).await;

                // reconciliation
                reconciliation::reconcile_token_spends(&mut dag);

                // export state for debugging
                dag.export_state();

                // checkpoint
                if last_checkpoint.elapsed() >= checkpoint_interval {
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
                            let block = finalize_block(tx_ids.clone(), &validators);
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
                                    .bind(block.id)
                                    .bind(block_json)
                                    .execute(&mut tx_sql)
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
                                            .bind(txid)
                                            .bind(block.id)
                                            .execute(&mut tx_sql)
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
                        }
                    }

                    last_checkpoint = Instant::now();
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    Ok(())
}
