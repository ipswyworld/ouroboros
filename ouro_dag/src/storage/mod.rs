// Storage abstraction layer - supports both PostgreSQL and RocksDB
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub mod postgres;
pub mod rocksdb_store;

// Re-exports
pub use postgres::PostgresStorage;
pub use rocksdb_store::RocksDbStorage;

/// Block data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub block_id: Uuid,
    pub block_height: u64,
    pub parent_ids: Vec<Uuid>,
    pub merkle_root: String,
    pub timestamp: i64,
    pub tx_count: i32,
    pub block_bytes: Vec<u8>,
    pub signer: Option<String>,
    pub signature: Option<String>,
}

/// Transaction data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub tx_id: Uuid,
    pub tx_hash: String,
    pub sender: Option<String>,
    pub recipient: Option<String>,
    pub payload: serde_json::Value,
    pub status: String,
    pub included_in_block: Option<Uuid>,
}

/// Balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub address: String,
    pub amount: i64,
}

/// Storage trait - implemented by both PostgreSQL and RocksDB backends
#[async_trait]
pub trait Storage: Send + Sync {
    // Block operations
    async fn save_block(&self, block: Block) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_block(&self, height: u64) -> Result<Option<Block>, Box<dyn std::error::Error>>;
    async fn get_block_by_id(&self, block_id: Uuid) -> Result<Option<Block>, Box<dyn std::error::Error>>;
    async fn get_latest_height(&self) -> Result<u64, Box<dyn std::error::Error>>;

    // Transaction operations
    async fn save_transaction(&self, tx: Transaction) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_transaction(&self, tx_hash: &str) -> Result<Option<Transaction>, Box<dyn std::error::Error>>;
    async fn get_transactions_by_block(&self, block_id: Uuid) -> Result<Vec<Transaction>, Box<dyn std::error::Error>>;

    // Balance operations
    async fn get_balance(&self, address: &str) -> Result<i64, Box<dyn std::error::Error>>;
    async fn update_balance(&self, address: &str, amount: i64) -> Result<(), Box<dyn std::error::Error>>;

    // Mempool operations
    async fn add_to_mempool(&self, tx: Transaction) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_mempool_txs(&self, limit: usize) -> Result<Vec<Transaction>, Box<dyn std::error::Error>>;
    async fn remove_from_mempool(&self, tx_hash: &str) -> Result<(), Box<dyn std::error::Error>>;

    // State operations
    async fn get_validator_set(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    async fn save_validator_set(&self, validators: Vec<String>) -> Result<(), Box<dyn std::error::Error>>;
}

/// Storage mode enum
#[derive(Debug, Clone)]
pub enum StorageMode {
    PostgreSQL,
    RocksDB,
}

impl StorageMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rocksdb" | "rocks" | "light" => StorageMode::RocksDB,
            _ => StorageMode::PostgreSQL,
        }
    }
}

/// Create storage backend based on mode
pub async fn create_storage(
    mode: StorageMode,
    db_path: Option<String>,
) -> Result<Arc<dyn Storage>, Box<dyn std::error::Error>> {
    match mode {
        StorageMode::PostgreSQL => {
            let db_url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://ouro:ouro_pass@localhost:5432/ouro_db".to_string());
            let storage = PostgresStorage::new(&db_url).await?;
            Ok(Arc::new(storage))
        }
        StorageMode::RocksDB => {
            let path = db_path.unwrap_or_else(|| {
                std::env::var("ROCKSDB_PATH")
                    .unwrap_or_else(|_| "./data/rocksdb".to_string())
            });
            let storage = RocksDbStorage::new(&path)?;
            Ok(Arc::new(storage))
        }
    }
}
