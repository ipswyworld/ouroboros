// src/node_metrics.rs
// Node metrics tracking and rewards calculation system

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Node contribution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_address: String,
    pub blocks_proposed: i64,
    pub blocks_validated: i64,
    pub transactions_processed: i64,
    pub uptime_seconds: i64,
    pub first_seen: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub total_rewards: i64,
}

/// Reward history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardHistoryEntry {
    pub id: i32,
    pub node_address: String,
    pub reward_type: String,
    pub amount: i64,
    pub block_height: Option<i64>,
    pub awarded_at: DateTime<Utc>,
}

/// Reward configuration
#[derive(Debug, Clone)]
pub struct RewardConfig {
    pub block_proposal_reward: i64,
    pub block_validation_reward: i64,
    pub uptime_reward_per_hour: i64,
    pub min_uptime_for_reward: i64,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            block_proposal_reward: 20,
            block_validation_reward: 3,
            uptime_reward_per_hour: 1,  // Note: stored as integer, actual value is 1.5 in DB
            min_uptime_for_reward: 3600, // 1 hour
        }
    }
}

/// Increment block proposal count for a validator
pub async fn record_block_proposal(
    pool: &PgPool,
    validator_address: &str,
    block_height: i64,
    tx_count: i64,
) -> Result<()> {
    // Update or insert node metrics
    sqlx::query(
        r#"
        INSERT INTO node_metrics (node_address, blocks_proposed, transactions_processed, last_active)
        VALUES ($1, 1, $2, NOW())
        ON CONFLICT (node_address) DO UPDATE SET
            blocks_proposed = node_metrics.blocks_proposed + 1,
            transactions_processed = node_metrics.transactions_processed + $2,
            last_active = NOW()
        "#,
    )
    .bind(validator_address)
    .bind(tx_count)
    .execute(pool)
    .await
    .context("Failed to record block proposal")?;

    // Award reward
    let config = load_reward_config(pool).await?;
    award_reward(
        pool,
        validator_address,
        "block_proposal",
        config.block_proposal_reward,
        Some(block_height),
    )
    .await?;

    Ok(())
}

/// Increment block validation count for a validator (when they vote)
pub async fn record_block_validation(
    pool: &PgPool,
    validator_address: &str,
    block_height: i64,
) -> Result<()> {
    // Update or insert node metrics
    sqlx::query(
        r#"
        INSERT INTO node_metrics (node_address, blocks_validated, last_active)
        VALUES ($1, 1, NOW())
        ON CONFLICT (node_address) DO UPDATE SET
            blocks_validated = node_metrics.blocks_validated + 1,
            last_active = NOW()
        "#,
    )
    .bind(validator_address)
    .execute(pool)
    .await
    .context("Failed to record block validation")?;

    // Award reward
    let config = load_reward_config(pool).await?;
    award_reward(
        pool,
        validator_address,
        "block_validation",
        config.block_validation_reward,
        Some(block_height),
    )
    .await?;

    Ok(())
}

/// Update node uptime (call periodically, e.g., every minute)
pub async fn update_node_uptime(
    pool: &PgPool,
    validator_address: &str,
    uptime_delta_seconds: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO node_metrics (node_address, uptime_seconds, last_active)
        VALUES ($1, $2, NOW())
        ON CONFLICT (node_address) DO UPDATE SET
            uptime_seconds = node_metrics.uptime_seconds + $2,
            last_active = NOW()
        "#,
    )
    .bind(validator_address)
    .bind(uptime_delta_seconds)
    .execute(pool)
    .await
    .context("Failed to update node uptime")?;

    Ok(())
}

/// Award uptime rewards (call periodically, e.g., every hour)
pub async fn award_uptime_rewards(pool: &PgPool) -> Result<()> {
    let config = load_reward_config(pool).await?;

    // Get all nodes that meet minimum uptime requirement
    let nodes: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT node_address, uptime_seconds
        FROM node_metrics
        WHERE uptime_seconds >= $1
        AND last_active > NOW() - INTERVAL '1 hour'
        "#,
    )
    .bind(config.min_uptime_for_reward)
    .fetch_all(pool)
    .await?;

    for (node_address, uptime_seconds) in nodes {
        let hours = uptime_seconds / 3600;
        let reward = hours * config.uptime_reward_per_hour;

        if reward > 0 {
            award_reward(pool, &node_address, "uptime_bonus", reward, None).await?;
        }
    }

    Ok(())
}

/// Award reward to a node and update total rewards
async fn award_reward(
    pool: &PgPool,
    node_address: &str,
    reward_type: &str,
    amount: i64,
    block_height: Option<i64>,
) -> Result<()> {
    // Insert reward history
    sqlx::query(
        r#"
        INSERT INTO rewards_history (node_address, reward_type, amount, block_height)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(node_address)
    .bind(reward_type)
    .bind(amount)
    .bind(block_height)
    .execute(pool)
    .await
    .context("Failed to insert reward history")?;

    // Update total rewards in node_metrics
    sqlx::query(
        r#"
        UPDATE node_metrics
        SET total_rewards = total_rewards + $1
        WHERE node_address = $2
        "#,
    )
    .bind(amount)
    .bind(node_address)
    .execute(pool)
    .await
    .context("Failed to update total rewards")?;

    Ok(())
}

/// Load reward configuration from database
async fn load_reward_config(pool: &PgPool) -> Result<RewardConfig> {
    let rows: Vec<(String, i64)> = sqlx::query_as("SELECT key, value FROM reward_config")
        .fetch_all(pool)
        .await?;

    let mut config = RewardConfig::default();

    for (key, value) in rows {
        match key.as_str() {
            "block_proposal_reward" => config.block_proposal_reward = value,
            "block_validation_reward" => config.block_validation_reward = value,
            "uptime_reward_per_hour" => config.uptime_reward_per_hour = value,
            "min_uptime_for_reward" => config.min_uptime_for_reward = value,
            _ => {}
        }
    }

    Ok(config)
}

/// Get node metrics for a specific validator
pub async fn get_node_metrics(pool: &PgPool, node_address: &str) -> Result<Option<NodeMetrics>> {
    let result = sqlx::query_as::<_, (
        String,
        i64,
        i64,
        i64,
        i64,
        DateTime<Utc>,
        DateTime<Utc>,
        i64,
    )>(
        r#"
        SELECT node_address, blocks_proposed, blocks_validated, transactions_processed,
               uptime_seconds, first_seen, last_active, total_rewards
        FROM node_metrics
        WHERE node_address = $1
        "#,
    )
    .bind(node_address)
    .fetch_optional(pool)
    .await?;

    Ok(result.map(
        |(
            node_address,
            blocks_proposed,
            blocks_validated,
            transactions_processed,
            uptime_seconds,
            first_seen,
            last_active,
            total_rewards,
        )| NodeMetrics {
            node_address,
            blocks_proposed,
            blocks_validated,
            transactions_processed,
            uptime_seconds,
            first_seen,
            last_active,
            total_rewards,
        },
    ))
}

/// Get leaderboard (top N validators by total rewards)
pub async fn get_leaderboard(pool: &PgPool, limit: i64) -> Result<Vec<NodeMetrics>> {
    let rows = sqlx::query_as::<_, (
        String,
        i64,
        i64,
        i64,
        i64,
        DateTime<Utc>,
        DateTime<Utc>,
        i64,
    )>(
        r#"
        SELECT node_address, blocks_proposed, blocks_validated, transactions_processed,
               uptime_seconds, first_seen, last_active, total_rewards
        FROM node_metrics
        ORDER BY total_rewards DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(
                node_address,
                blocks_proposed,
                blocks_validated,
                transactions_processed,
                uptime_seconds,
                first_seen,
                last_active,
                total_rewards,
            )| NodeMetrics {
                node_address,
                blocks_proposed,
                blocks_validated,
                transactions_processed,
                uptime_seconds,
                first_seen,
                last_active,
                total_rewards,
            },
        )
        .collect())
}

/// Get reward history for a specific validator
pub async fn get_reward_history(
    pool: &PgPool,
    node_address: &str,
    limit: i64,
) -> Result<Vec<RewardHistoryEntry>> {
    let rows = sqlx::query_as::<_, (i32, String, String, i64, Option<i64>, DateTime<Utc>)>(
        r#"
        SELECT id, node_address, reward_type, amount, block_height, awarded_at
        FROM rewards_history
        WHERE node_address = $1
        ORDER BY awarded_at DESC
        LIMIT $2
        "#,
    )
    .bind(node_address)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, node_address, reward_type, amount, block_height, awarded_at)| {
                RewardHistoryEntry {
                    id,
                    node_address,
                    reward_type,
                    amount,
                    block_height,
                    awarded_at,
                }
            },
        )
        .collect())
}
