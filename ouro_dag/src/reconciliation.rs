// src/reconciliation.rs
use crate::dag::dag::DAG;
use crate::dag::transaction::Transaction;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize)]
struct TokenSpend {
    sender: String,
    recipient: String,
    amount: i64,
    timestamp: String,
}

pub fn reconcile_token_spends(dag: &mut DAG) {
    let path = Path::new("token_spends.json");
    if !path.exists() {
        return;
    }

    let data = match fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("reconciliation: failed to read token_spends.json: {}", e);
            return;
        }
    };

    let parsed: Vec<TokenSpend> = match serde_json::from_str(&data) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("reconciliation: failed to parse token_spends.json: {}", e);
            return;
        }
    };

    let existing_ids: HashSet<_> = dag.transactions.keys().cloned().collect();

    for spend in parsed {
        let amount_u64 = if spend.amount >= 0 {
            spend.amount as u64
        } else {
            eprintln!("reconciliation: ignoring negative amount in token_spend from {}", spend.sender);
            continue;
        };

        let timestamp = match DateTime::parse_from_rfc3339(&spend.timestamp) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(e) => {
                eprintln!("reconciliation: bad timestamp {}: {}", spend.timestamp, e);
                continue;
            }
        };

        let txn = Transaction {
            id: Uuid::new_v4(),
            sender: spend.sender.clone(),
            recipient: spend.recipient.clone(),
            amount: amount_u64,
            timestamp,
            parents: vec![],
            signature: "token-spend".into(),
            public_key: "none".into(),
            payload: None,
        };

        if !existing_ids.contains(&txn.id) {
            if let Err(e) = dag.add_transaction(txn) {
                eprintln!("reconciliation: failed to add txn: {}", e);
            }
        }
    }

    if let Err(e) = fs::remove_file(path) {
        eprintln!("reconciliation: failed to remove token_spends.json: {}", e);
    }
}
