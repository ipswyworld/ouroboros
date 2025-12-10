// src/vm.rs
// Minimal native VM for Ouroboros: supports simple native contracts (SBT).
//
// This file contains:
// - TxResult struct
// - Contract trait
// - SBTContract native implementation
// - execute_contracts function used by main during block finalization.

use crate::sled_storage::{RocksDb, put_str, get_str};
use crate::dag::transaction::Transaction;
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Represents the result of a transaction's execution in the VM.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TxResult {
    pub tx_id: Uuid,
    pub tx_hash: String, // Using tx.id as the hash/identifier
    pub status: String, // "ok" or "failed"
    pub result: Option<JsonValue>, // JSON result from contract
}

/// Contract trait for native contracts.
pub trait Contract {
    fn name(&self) -> &'static str;
    /// Execute a contract for a single transaction.
    /// Returns JSON result on success or Err(String) on failure.
    fn execute(&self, db: &RocksDb, tx: &Transaction) -> Result<JsonValue, String>;
}

/// Simple SBT contract implementation.
pub struct SBTContract;

impl Contract for SBTContract {
    fn name(&self) -> &'static str {
        "sbt"
    }

    fn execute(&self, db: &RocksDb, tx: &Transaction) -> Result<JsonValue, String> {
        // Expect tx.payload JSON string (some transactions may not have payloads)
        let payload_str = tx.payload.as_ref().ok_or("missing payload")?;
        let payload: JsonValue = serde_json::from_str(payload_str).map_err(|e| e.to_string())?;
        let op = payload.get("op").and_then(|v| v.as_str()).ok_or("missing op")?;

        match op {
            "mint" => {
                let sbt_id = payload
                    .get("sbt_id")
                    .and_then(|v| v.as_str())
                    .ok_or("missing sbt_id")?;
                let meta = payload.get("meta").cloned().unwrap_or(JsonValue::Null);
                let sbt_key = format!("sbt:{}", sbt_id);
                let store_obj = serde_json::json!({
                    "sbt_id": sbt_id,
                    "issuer": tx.sender,
                    "meta": meta,
                    "mint_tx": tx.id.to_string(),
                    "timestamp": tx.timestamp.to_rfc3339()
                });
                // store as JSON string
                put_str(db, &sbt_key, &store_obj)
                    .map_err(|e| format!("failed to store sbt: {}", e))?;
                Ok(serde_json::json!({"status":"minted","sbt_id": sbt_id}))
            }
            "revoke" => {
                let sbt_id = payload
                    .get("sbt_id")
                    .and_then(|v| v.as_str())
                    .ok_or("missing sbt_id")?;
                let sbt_key = format!("sbt:{}", sbt_id);
                if let Ok(Some(mut existing)) = get_str::<JsonValue>(db, &sbt_key) {
                    existing["revoked"] = serde_json::json!(true);
                    existing["revoked_by"] = serde_json::json!(tx.sender.clone());
                    existing["revoked_at"] = serde_json::json!(tx.timestamp.to_rfc3339());
                    put_str(db, &sbt_key, &existing)
                        .map_err(|e| format!("failed to update sbt: {}", e))?;
                    Ok(serde_json::json!({"status":"revoked","sbt_id": sbt_id}))
                } else {
                    Err("sbt not found".into())
                }
            }
            _ => Err("unsupported op".into()),
        }
    }
}

/// Execute contracts found in `txs` (called during block formation).
/// Returns a Vec<TxResult> describing each tx's inclusion/result.
/// If you want strict policy (abort block on any contract error) you can propagate Err.
/// Here we collect per-tx results and return Err only for fatal VM-level issues.
pub fn execute_contracts(db: &RocksDb, txs: &[Transaction]) -> Result<Vec<TxResult>, String> {
    let sbt = SBTContract;
    let mut results: Vec<TxResult> = Vec::with_capacity(txs.len());

    for tx in txs {
        let tx_hash = tx.id.to_string();
        let tx_id = tx.id;

        if let Some(payload_str) = &tx.payload {
            let parsed = match serde_json::from_str::<JsonValue>(payload_str) {
                Ok(p) => p,
                Err(e) => {
                    // deterministic parse failure -> mark tx failed but continue
                    let tr = TxResult {
                        tx_id,
                        tx_hash: tx_hash.clone(),
                        status: "failed".to_string(),
                        result: Some(serde_json::json!({"error": format!("parse error: {}", e)})),
                    };
                    results.push(tr);
                    continue;
                }
            };

            if let Some(contract_name) = parsed.get("contract").and_then(|v| v.as_str()) {
                let res = match contract_name {
                    "sbt" => sbt.execute(db, tx),
                    other => Err(format!("unknown native contract: {}", other)),
                };

                match res {
                    Ok(value) => {
                        // persist a receipt for this tx (receipt:<tx_id>)
                        let receipt_key = format!("receipt:{}", tx.id);
                        if let Err(e) = put_str(db, &receipt_key, &value) {
                            return Err(format!("failed to persist receipt for tx {}: {}", tx.id, e));
                        }

                        let tr = TxResult {
                            tx_id,
                            tx_hash: tx_hash.clone(),
                            status: "ok".to_string(),
                            result: Some(value),
                        };
                        results.push(tr);
                    }
                    Err(errstr) => {
                        let tr = TxResult {
                            tx_id,
                            tx_hash: tx_hash.clone(),
                            status: "failed".to_string(),
                            result: Some(serde_json::json!({"error": errstr})),
                        };
                        results.push(tr);
                        // policy: continue building block (tx will be marked failed)
                    }
                }
                continue;
            }
        }

        // No contract => basic accepted tx
        let tr = TxResult {
            tx_id,
            tx_hash: tx_hash.clone(),
            status: "ok".to_string(),
            result: None,
        };

        // persist a minimal receipt
        let key = format!("receipt:{}", tx.id);
        let r = serde_json::json!({
            "tx_id": tx.id.to_string(),
            "sender": tx.sender,
            "recipient": tx.recipient,
            "amount": tx.amount,
            "status": "ok"
        });
        if let Err(e) = put_str(db, &key, &r) {
            return Err(format!("failed to persist receipt for tx {}: {}", tx.id, e));
        }

        results.push(tr);
    }

    Ok(results)
}
