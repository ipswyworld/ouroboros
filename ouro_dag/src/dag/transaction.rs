// src/dag/transaction.rs
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub parents: Vec<Uuid>,
    pub signature: String,
    pub public_key: String,

    /// Optional JSON payload for contract calls / extended semantics.
    /// Example: {"contract":"sbt","op":"mint","sbt_id":"abc","meta":{...}}
    pub payload: Option<String>,
}
