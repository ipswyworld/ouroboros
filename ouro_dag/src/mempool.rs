// src/mempool.rs
use crate::storage::{RocksDb, put_str, delete, iter_prefix_kv};
use crate::dag::transaction::Transaction;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Persistent, simple mempool with in-memory queue + sled persistence per tx.
#[derive(Clone)]
pub struct Mempool {
    db: RocksDb,
    inner: Arc<Mutex<Vec<Transaction>>>,
}

impl Mempool {
    /// Load persisted mempool entries from DB and return a Mempool instance.
    pub fn new(db: RocksDb) -> Self {
        let mut in_mem = Vec::new();
        if let Ok(list) = iter_prefix_kv::<Transaction>(&db, "mempool:") {
            for (_k, tx) in list {
                in_mem.push(tx);
            }
        }
        Mempool { db, inner: Arc::new(Mutex::new(in_mem)) }
    }

    /// Add a transaction: persist first, then push to in-memory queue.
    pub fn add_tx(&self, tx: &Transaction) -> Result<(), String> {
        let key = format!("mempool:{}", tx.id);
        put_str(&self.db, &key, tx)?;
        let mut guard = self.inner.lock().unwrap();
        guard.push(tx.clone());
        Ok(())
    }

    /// Check if tx exists in mempool (in-memory check)
    pub fn contains(&self, id: &Uuid) -> Result<bool, String> {
        let guard = self.inner.lock().unwrap();
        Ok(guard.iter().any(|t| &t.id == id))
    }

    /// Return the number of txs currently in mempool
    pub fn len(&self) -> usize {
        let guard = self.inner.lock().unwrap();
        guard.len()
    }

    /// Pop up to `n` txs for block creation and remove them from DB
    pub fn pop_for_block(&self, n: usize) -> Result<Vec<Transaction>, String> {
        let mut guard = self.inner.lock().unwrap();
        let count = std::cmp::min(n, guard.len());
        let mut take = Vec::with_capacity(count);
        for _ in 0..count {
            // remove(0) returns a Transaction directly
            let tx = guard.remove(0);
            take.push(tx);
        }
        // remove from DB
        for tx in &take {
            let key = format!("mempool:{}", tx.id);
            let _ = delete(&self.db, key);
        }
        Ok(take)
    }
}
