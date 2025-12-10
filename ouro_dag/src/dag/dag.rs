use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use super::transaction::Transaction;
use super::validation::validate_transaction;
use std::fs::File;
use std::io::Write;
use serde::Serialize;
use crate::sled_storage::{RocksDb, put, iter_prefix};

pub struct DAG {
    pub transactions: HashMap<Uuid, Transaction>,
    pub db: RocksDb,
}

#[derive(Serialize)]
struct ExportedTxn {
    sender: String,
    recipient: String,
    amount: u64,
}

#[derive(Serialize)]
struct ExportedState {
    balances: HashMap<String, u64>,
    transactions: Vec<ExportedTxn>,
}

impl DAG {
    // New constructor requires a RocksDb (sled-backed) handle.
    pub fn new(db: RocksDb) -> Self {
        let mut transactions = HashMap::new();

        // Load persisted transactions from storage using the prefix "txn:"
        if let Ok(stored) = iter_prefix::<Transaction>(&db, b"txn:") {
            for txn in stored {
                transactions.insert(txn.id, txn);
            }
            println!("Loaded {} transactions from DB", transactions.len());
        } else {
            println!("No persisted transactions found or failed to read prefix");
        }

        DAG {
            transactions,
            db,
        }
    }

    pub fn add_transaction(&mut self, txn: Transaction) -> Result<(), String> {
        let existing_ids: HashSet<_> = self.transactions.keys().cloned().collect();
        validate_transaction(&txn, &existing_ids)?;

        // Persist to DB as JSON under key "txn:<uuid>"
        let key = format!("txn:{}", txn.id);
        put(&self.db, key.into_bytes(), &txn)?;

        // Insert into in-memory cache
        self.transactions.insert(txn.id, txn);
        Ok(())
    }

    pub fn print_dag(&self) {
        for (id, txn) in &self.transactions {
            println!(
                "Txn ID: {}, From: {}, To: {}, Amount: {}, Parents: {:?}",
                id, txn.sender, txn.recipient, txn.amount, txn.parents
            );
        }
    }

    pub fn export_state(&self) {
        let mut balances = HashMap::new();
        let mut transactions = vec![];

        for txn in self.transactions.values() {
            let sender_balance = balances.entry(txn.sender.clone()).or_insert(0u64);
            *sender_balance = sender_balance.saturating_sub(txn.amount);
            *balances.entry(txn.recipient.clone()).or_insert(0u64) += txn.amount;

            transactions.push(ExportedTxn {
                sender: txn.sender.clone(),
                recipient: txn.recipient.clone(),
                amount: txn.amount,
            });
        }

        let state = ExportedState {
            balances,
            transactions,
        };

        let json = serde_json::to_string_pretty(&state).unwrap();
        let mut file = File::create("dag_state.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}


// src/dag/dag.rs  (append)
use sled;

use anyhow::Result;
use std::path::PathBuf;
use crate::bft::consensus::Block;

/// Initialize/obtain sled DB for blocks under repo ./sled_data/blocks.db
fn sled_db() -> Result<sled::Db> {
    let base_path = std::env::var("ROCKSDB_PATH")
        .or_else(|_| std::env::var("SLED_PATH"))
        .unwrap_or_else(|_| "sled_data".into());
    let mut p = PathBuf::from(&base_path);
    p.push("blocks.db");
    std::fs::create_dir_all(p.parent().unwrap())?;
    let db = sled::open(p)?;
    Ok(db)
}

/// Insert block into sled store. Returns assigned block id.
pub async fn insert_block(proposer: &str, _view: u64, tx_ids: Vec<Uuid>) -> Result<Uuid> {
    let db = sled_db()?;
    let b = Block::new(proposer, tx_ids);
    let id = b.id;
    let key = id.as_bytes();
    let v = serde_json::to_vec(&b)?;
    db.insert(key, v)?;
    db.flush()?;
    Ok(id)
}

/// Backwards-compatible name used earlier by some stubs.
pub async fn insert_block_stub(tx_ids: Vec<Uuid>, proposer: &str, view: u64) -> Result<Uuid> {
    insert_block(proposer, view, tx_ids).await
}

/// Get tx ids for a block (empty vector if not found).
pub async fn get_txids_for_block(block_id: Uuid) -> Result<Vec<Uuid>> {
    let db = sled_db()?;
    if let Some(v) = db.get(block_id.as_bytes())? {
        let b: Block = serde_json::from_slice(&v)?;
        Ok(b.tx_ids)
    } else {
        Ok(vec![])
    }
}

use std::io;

/// Attempt to load a transaction by id from the store (rocksdb/sled).
/// This is a simple helper used by reconciliation::finalize_block.
/// You must adapt the key-space to match how transactions are stored.
pub fn get_transaction(txid: Uuid) -> Result<Transaction, io::Error> {
    // Example implementation for sled-backed tx store.
    // Replace path / db access with your project's actual store handle.
    let base_path = std::env::var("ROCKSDB_PATH")
        .or_else(|_| std::env::var("SLED_PATH"))
        .unwrap_or_else(|_| "sled_data".into());
    let mut p = PathBuf::from(&base_path);
    p.push("mempool_tx");
    let db = sled::open(p)?;
    let key = txid.as_bytes();
    match db.get(key)? {
        Some(val) => {
            // assume serialized via bincode or serde_json; try serde_json first
            let txn: Transaction = serde_json::from_slice(&val)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("deserialize err: {}", e)))?;
            Ok(txn)
        }
        None => Err(io::Error::new(io::ErrorKind::NotFound, "tx not found"))
    }
}