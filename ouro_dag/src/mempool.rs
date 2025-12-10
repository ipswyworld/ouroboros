// src/mempool.rs
use crate::sled_storage::{RocksDb, put, iter_prefix};
use crate::dag::transaction::Transaction;
use uuid::Uuid;
use std::io::{Result as IoResult, Error as IoError, ErrorKind};
use std::sync::Arc;
use anyhow;
use once_cell::sync::OnceCell;

/// Global mempool instance (initialized once during startup)
static GLOBAL_MEMPOOL: OnceCell<Arc<Mempool>> = OnceCell::new();

/// Initialize the global mempool instance.
/// This should be called once during node startup with the storage handle.
pub fn init_global_mempool(db: RocksDb) {
    let mempool = Mempool::new(db);
    let _ = GLOBAL_MEMPOOL.set(Arc::new(mempool));
}

/// Select up to `limit` transactions from the global mempool for inclusion in a block.
/// Returns Vec of transaction UUIDs.
///
/// This function is called by consensus when proposing a new block.
/// It retrieves transactions from the mempool without removing them (removal happens
/// after block finalization).
pub async fn select_transactions(limit: usize) -> anyhow::Result<Vec<Uuid>> {
    // Get global mempool instance
    let mempool = GLOBAL_MEMPOOL
        .get()
        .ok_or_else(|| anyhow::anyhow!("Mempool not initialized - call init_global_mempool() first"))?;

    // Pop transactions from mempool (async-safe via tokio::task::spawn_blocking)
    let mempool_clone = Arc::clone(mempool);
    let txs = tokio::task::spawn_blocking(move || {
        mempool_clone.pop_for_block(limit)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
    .map_err(|e| anyhow::anyhow!("Mempool pop error: {}", e))?;

    // Extract transaction IDs
    let tx_ids: Vec<Uuid> = txs.iter().map(|tx| tx.id).collect();

    log::info!("Selected {} transactions from mempool (limit: {})", tx_ids.len(), limit);

    Ok(tx_ids)
}


/// In-memory / on-disk mempool wrapper.
/// TODO: make operations fully asynchronous if needed and add proper eviction/prioritization.
pub struct Mempool {
    pub db: RocksDb,
}

impl Mempool {
    /// Construct a mempool bound to a RocksDb handle from crate::storage.
    pub fn new(db: RocksDb) -> Self {
        Self { db }
    }

    /// Add a transaction to the mempool and persist it under key "mempool:<uuid>".
    /// Returns std::io::Result for backward compatibility with earlier code.
    pub fn add_tx(&self, txn: &Transaction) -> IoResult<()> {
        let key = format!("mempool:{}", txn.id);
        // `put` is a thin wrapper in crate::storage used in dag.rs. Propagate errors as IoError.
        put(&self.db, key.into_bytes(), txn).map_err(|e| IoError::new(ErrorKind::Other, format!("db put error: {}", e)))?;
        Ok(())
    }

    /// Return up to `limit` transactions from mempool, prioritized by fee (highest first).
    /// Also implements TTL-based eviction (transactions older than 24 hours are skipped).
    pub fn pop_for_block(&self, limit: usize) -> IoResult<Vec<Transaction>> {
        // Read all transactions from mempool
        let mut txs = match iter_prefix::<Transaction>(&self.db, b"mempool:") {
            Ok(items) => items,
            Err(e) => return Err(IoError::new(ErrorKind::Other, format!("iter_prefix error: {}", e))),
        };

        // TTL-based eviction: remove transactions older than 24 hours
        let now = chrono::Utc::now();
        let ttl = chrono::Duration::hours(24);
        txs.retain(|tx| now.signed_duration_since(tx.timestamp) < ttl);

        // Sort by fee (descending - highest fee first), then by timestamp (oldest first for fairness)
        txs.sort_by(|a, b| {
            b.fee.cmp(&a.fee)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });

        // Take up to limit transactions
        let selected = txs.into_iter().take(limit).collect();

        Ok(selected)
    }


}


