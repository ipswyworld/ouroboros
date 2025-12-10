// PostgreSQL storage implementation - wraps existing functionality
use super::{Balance, Block, Storage, Transaction};
use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use uuid::Uuid;

/// PostgreSQL storage backend
pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    /// Create new PostgreSQL storage
    pub async fn new(database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(100)
            .acquire_timeout(Duration::from_secs(10))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(1800))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Get pool reference (for backwards compatibility with existing code)
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl Storage for PostgresStorage {
    // Block operations
    async fn save_block(&self, block: Block) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"
            INSERT INTO blocks (
                block_id, block_height, parent_ids, merkle_root,
                timestamp, tx_count, block_bytes, signer, signature
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (block_id) DO NOTHING
            "#,
        )
        .bind(block.block_id)
        .bind(block.block_height as i64)
        .bind(&block.parent_ids)
        .bind(&block.merkle_root)
        .bind(chrono::DateTime::from_timestamp(block.timestamp, 0))
        .bind(block.tx_count)
        .bind(&block.block_bytes)
        .bind(&block.signer)
        .bind(&block.signature)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_block(&self, height: u64) -> Result<Option<Block>, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, (Uuid, i64, Vec<Uuid>, String, chrono::DateTime<chrono::Utc>, i32, Vec<u8>, Option<String>, Option<String>)>(
            "SELECT block_id, block_height, parent_ids, merkle_root, timestamp, tx_count, block_bytes, signer, signature FROM blocks WHERE block_height = $1"
        )
        .bind(height as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(block_id, block_height, parent_ids, merkle_root, timestamp, tx_count, block_bytes, signer, signature)| {
            Block {
                block_id,
                block_height: block_height as u64,
                parent_ids,
                merkle_root,
                timestamp: timestamp.timestamp(),
                tx_count,
                block_bytes,
                signer,
                signature,
            }
        }))
    }

    async fn get_block_by_id(&self, block_id: Uuid) -> Result<Option<Block>, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, (Uuid, i64, Vec<Uuid>, String, chrono::DateTime<chrono::Utc>, i32, Vec<u8>, Option<String>, Option<String>)>(
            "SELECT block_id, block_height, parent_ids, merkle_root, timestamp, tx_count, block_bytes, signer, signature FROM blocks WHERE block_id = $1"
        )
        .bind(block_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(block_id, block_height, parent_ids, merkle_root, timestamp, tx_count, block_bytes, signer, signature)| {
            Block {
                block_id,
                block_height: block_height as u64,
                parent_ids,
                merkle_root,
                timestamp: timestamp.timestamp(),
                tx_count,
                block_bytes,
                signer,
                signature,
            }
        }))
    }

    async fn get_latest_height(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(block_height), 0) FROM blocks"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(h,)| h as u64).unwrap_or(0))
    }

    // Transaction operations
    async fn save_transaction(&self, tx: Transaction) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"
            INSERT INTO transactions (
                tx_id, tx_hash, sender, recipient, payload, status, included_in_block
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (tx_id) DO NOTHING
            "#,
        )
        .bind(tx.tx_id)
        .bind(&tx.tx_hash)
        .bind(&tx.sender)
        .bind(&tx.recipient)
        .bind(&tx.payload)
        .bind(&tx.status)
        .bind(&tx.included_in_block)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_transaction(&self, tx_hash: &str) -> Result<Option<Transaction>, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>, serde_json::Value, String, Option<Uuid>)>(
            "SELECT tx_id, tx_hash, sender, recipient, payload, status, included_in_block FROM transactions WHERE tx_hash = $1"
        )
        .bind(tx_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(tx_id, tx_hash, sender, recipient, payload, status, included_in_block)| {
            Transaction {
                tx_id,
                tx_hash,
                sender,
                recipient,
                payload,
                status,
                included_in_block,
            }
        }))
    }

    async fn get_transactions_by_block(&self, block_id: Uuid) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let results = sqlx::query_as::<_, (Uuid, String, Option<String>, Option<String>, serde_json::Value, String, Option<Uuid>)>(
            "SELECT tx_id, tx_hash, sender, recipient, payload, status, included_in_block FROM transactions WHERE included_in_block = $1"
        )
        .bind(block_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results.into_iter().map(|(tx_id, tx_hash, sender, recipient, payload, status, included_in_block)| {
            Transaction {
                tx_id,
                tx_hash,
                sender,
                recipient,
                payload,
                status,
                included_in_block,
            }
        }).collect())
    }

    // Balance operations
    async fn get_balance(&self, address: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(amount, 0) FROM balances WHERE address = $1"
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(amount,)| amount).unwrap_or(0))
    }

    async fn update_balance(&self, address: &str, amount: i64) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"
            INSERT INTO balances (address, amount)
            VALUES ($1, $2)
            ON CONFLICT (address) DO UPDATE SET amount = $2
            "#,
        )
        .bind(address)
        .bind(amount)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Mempool operations
    async fn add_to_mempool(&self, tx: Transaction) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "INSERT INTO mempool_entries (tx_id, tx_hash, payload) VALUES ($1, $2, $3) ON CONFLICT (tx_id) DO NOTHING"
        )
        .bind(tx.tx_id)
        .bind(&tx.tx_hash)
        .bind(&tx.payload)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_mempool_txs(&self, limit: usize) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
        let results = sqlx::query_as::<_, (Uuid, String, serde_json::Value)>(
            "SELECT tx_id, tx_hash, payload FROM mempool_entries ORDER BY received_at LIMIT $1"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(results.into_iter().map(|(tx_id, tx_hash, payload)| {
            Transaction {
                tx_id,
                tx_hash,
                sender: None,
                recipient: None,
                payload,
                status: "pending".to_string(),
                included_in_block: None,
            }
        }).collect())
    }

    async fn remove_from_mempool(&self, tx_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query("DELETE FROM mempool_entries WHERE tx_hash = $1")
            .bind(tx_hash)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // State operations
    async fn get_validator_set(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let results: Vec<(String,)> = sqlx::query_as(
            "SELECT validator_address FROM validator_registry WHERE active = true"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results.into_iter().map(|(addr,)| addr).collect())
    }

    async fn save_validator_set(&self, validators: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // First, mark all as inactive
        sqlx::query("UPDATE validator_registry SET active = false")
            .execute(&self.pool)
            .await?;

        // Then insert/update active validators
        for validator in validators {
            sqlx::query(
                r#"
                INSERT INTO validator_registry (validator_address, active)
                VALUES ($1, true)
                ON CONFLICT (validator_address) DO UPDATE SET active = true
                "#,
            )
            .bind(&validator)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
