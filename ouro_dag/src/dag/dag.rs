use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use super::transaction::Transaction;
use super::validation::validate_transaction;
use std::fs::File;
use std::io::Write;
use serde::Serialize;
use crate::storage::{RocksDb, put, iter_prefix};

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
