// RocksDB storage implementation for lightweight nodes
use super::{Balance, Block, Storage, Transaction};
use async_trait::async_trait;
use rocksdb::{DB, Options};
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

/// RocksDB storage backend
pub struct RocksDbStorage {
    db: Arc<DB>,
}

impl RocksDbStorage {
    /// Create new RocksDB storage
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        let db = DB::open(&opts, path)?;

        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Helper: Serialize to bytes
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(value)?)
    }

    /// Helper: Deserialize from bytes
    fn deserialize<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(bytes)?)
    }
}

#[async_trait]
impl Storage for RocksDbStorage {
    // Block operations
    async fn save_block(&self, block: Block) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("block:height:{}", block.block_height);
        let value = Self::serialize(&block)?;
        self.db.put(key.as_bytes(), value)?;

        // Also index by block_id
        let id_key = format!("block:id:{}", block.block_id);
        let height_bytes = block.block_height.to_be_bytes();
        self.db.put(id_key.as_bytes(), height_bytes)?;

        // Update latest height
        self.db.put(b"state:latest_height", block.block_height.to_be_bytes())?;

        Ok(())
    }

    async fn get_block(&self, height: u64) -> Result<Option<Block>, Box<dyn std::error::Error>> {
        let key = format!("block:height:{}", height);

        match self.db.get(key.as_bytes())? {
            Some(bytes) => {
                let block = Self::deserialize(&bytes)?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    async fn get_block_by_id(&self, block_id: Uuid) -> Result<Option<Block>, Box<dyn std::error::Error>> {
        let id_key = format!("block:id:{}", block_id);

        match self.db.get(id_key.as_bytes())? {
            Some(height_bytes) => {
                let height = u64::from_be_bytes(height_bytes.try_into().unwrap_or([0u8; 8]));
                self.get_block(height).await
            }
            None => Ok(None),
        }
    }

    async fn get_latest_height(&self) -> Result<u64, Box<dyn std::error::Error>> {
        match self.db.get(b"state:latest_height")? {
            Some(bytes) => {
                let height = u64::from_be_bytes(bytes.try_into().unwrap_or([0u8; 8]));
                Ok(height)
            }
            None => Ok(0),
        }
    }

    // Transaction operations
    async fn save_transaction(&self, tx: Transaction) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("tx:hash:{}", tx.tx_hash);
        let value = Self::serialize(&tx)?;
        self.db.put(key.as_bytes(), value)?;

        // Index by tx_id
        let id_key = format!("tx:id:{}", tx.tx_id);
        self.db.put(id_key.as_bytes(), tx.tx_hash.as_bytes())?;

        // If included in block, index by block
        if let Some(block_id) = tx.included_in_block {
            let block_tx_key = format!("block:{}:tx:{}", block_id, tx.tx_hash);
            self.db.put(block_tx_key.as_bytes(), b"1")?;
        }

        Ok(())
    }

    async fn get_transaction(&self, tx_hash: &str) -> Result<Option<Transaction>, Box<dyn std::error::Error>> {
        let key = format!("tx:hash:{}", tx_hash);

        match self.db.get(key.as_bytes())? {
            Some(bytes) => {
                let tx = Self::deserialize(&bytes)?;
                Ok(Some(tx))
            }
            None => Ok(None),
        }
    }

    async fn get_transactions_by_block(&self, block_id: Uuid) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let prefix = format!("block:{}:tx:", block_id);
        let mut transactions = Vec::new();

        let iter = self.db.prefix_iterator(prefix.as_bytes());
        for item in iter {
            let (key, _) = item?;
            let key_str = String::from_utf8_lossy(&key);

            // Extract tx_hash from key: "block:{block_id}:tx:{tx_hash}"
            if let Some(tx_hash) = key_str.split(':').nth(3) {
                if let Some(tx) = self.get_transaction(tx_hash).await? {
                    transactions.push(tx);
                }
            }
        }

        Ok(transactions)
    }

    // Balance operations
    async fn get_balance(&self, address: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let key = format!("balance:{}", address);

        match self.db.get(key.as_bytes())? {
            Some(bytes) => {
                let balance = i64::from_be_bytes(bytes.try_into().unwrap_or([0u8; 8]));
                Ok(balance)
            }
            None => Ok(0),
        }
    }

    async fn update_balance(&self, address: &str, amount: i64) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("balance:{}", address);
        self.db.put(key.as_bytes(), amount.to_be_bytes())?;
        Ok(())
    }

    // Mempool operations
    async fn add_to_mempool(&self, tx: Transaction) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("mempool:{}", tx.tx_hash);
        let value = Self::serialize(&tx)?;
        self.db.put(key.as_bytes(), value)?;
        Ok(())
    }

    async fn get_mempool_txs(&self, limit: usize) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let prefix = b"mempool:";
        let mut transactions = Vec::new();

        let iter = self.db.prefix_iterator(prefix);
        for (i, item) in iter.enumerate() {
            if i >= limit {
                break;
            }

            let (_, value) = item?;
            let tx = Self::deserialize(&value)?;
            transactions.push(tx);
        }

        Ok(transactions)
    }

    async fn remove_from_mempool(&self, tx_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("mempool:{}", tx_hash);
        self.db.delete(key.as_bytes())?;
        Ok(())
    }

    // State operations
    async fn get_validator_set(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        match self.db.get(b"state:validators")? {
            Some(bytes) => {
                let validators: Vec<String> = serde_json::from_slice(&bytes)?;
                Ok(validators)
            }
            None => Ok(Vec::new()),
        }
    }

    async fn save_validator_set(&self, validators: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let value = serde_json::to_vec(&validators)?;
        self.db.put(b"state:validators", value)?;
        Ok(())
    }
}
