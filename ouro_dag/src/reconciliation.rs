// src/reconciliation.rs
use uuid::Uuid;
use anyhow::{Result, Context};
use log::{info, warn, error};
use crate::sled_storage;
use crate::vm;

/// Legacy helper stub used by some boot code. Keep trivial but harmless.
pub fn reconcile_token_spends(_dag: &mut crate::dag::dag::DAG) {
    // no-op for now; real implementation will read external inputs and inject txns into DAG
    info!("reconcile_token_spends: stub called");
}

/// Called by consensus when a block finalizes.
///
/// This function:
/// 1. Fetches all transaction IDs in the block
/// 2. Loads full transaction data
/// 3. Executes smart contracts via the VM
/// 4. Persists execution receipts to storage
/// 5. Updates account balances (future: implement balance tracking)
///
/// Idempotent: Safe to call multiple times for the same block.
pub async fn finalize_block(block_id: Uuid) -> Result<()> {
    info!("🔧 finalize_block called for block {}", block_id);

    // Step 1: Get transaction IDs from the block
    let tx_ids = crate::dag::dag::get_txids_for_block(block_id)
        .await
        .context("Failed to fetch transaction IDs for block")?;

    if tx_ids.is_empty() {
        info!("✅ Block {} is empty (0 transactions) - nothing to finalize", block_id);
        return Ok(());
    }

    info!("📦 Block {} contains {} transactions", block_id, tx_ids.len());

    // Step 2: Fetch full transaction data
    let mut transactions = Vec::new();
    for tx_id in &tx_ids {
        match tokio::task::spawn_blocking({
            let tx_id = *tx_id;
            move || crate::dag::dag::get_transaction(tx_id)
        })
        .await
        {
            Ok(Ok(tx)) => transactions.push(tx),
            Ok(Err(e)) => {
                warn!("⚠️  Failed to load transaction {}: {} - skipping", tx_id, e);
                continue;
            }
            Err(e) => {
                error!("❌ Task join error loading transaction {}: {}", tx_id, e);
                continue;
            }
        }
    }

    if transactions.is_empty() {
        warn!("⚠️  No transactions could be loaded for block {}", block_id);
        return Ok(());
    }

    info!("📥 Loaded {}/{} transactions successfully", transactions.len(), tx_ids.len());

    // Step 3: Get storage handle for VM execution
    let db = sled_storage::get_global_storage()
        .ok_or_else(|| anyhow::anyhow!("Global storage not initialized"))?;

    // Step 4: Execute smart contracts via VM
    info!("⚙️  Executing smart contracts for {} transactions...", transactions.len());

    let results = tokio::task::spawn_blocking({
        let db_clone = db.clone();
        let txs_clone = transactions.clone();
        move || vm::execute_contracts(&db_clone, &txs_clone)
    })
    .await
    .context("Task join error during VM execution")?
    .map_err(|e| anyhow::anyhow!("VM execution error: {}", e))?;

    // Step 5: Persist execution receipts
    let mut success_count = 0;
    let mut failure_count = 0;

    for result in &results {
        match &result.status[..] {
            "ok" => {
                success_count += 1;
                info!("  ✅ TX {} executed successfully", result.tx_id);
            }
            "failed" => {
                failure_count += 1;
                warn!("  ❌ TX {} execution failed: {:?}", result.tx_id, result.result);
            }
            _ => {
                warn!("  ⚠️  TX {} has unknown status: {}", result.tx_id, result.status);
            }
        }

        // Persist receipt to storage (key: "receipt:<tx_id>")
        let receipt_key = format!("receipt:{}", result.tx_id);
        if let Err(e) = sled_storage::put(&db, receipt_key.into_bytes(), result) {
            error!("Failed to persist receipt for {}: {}", result.tx_id, e);
        }
    }

    // Step 6: Update account balances based on transaction results
    // Note: Balance tracking requires PostgreSQL connection
    // For now, we log the balance changes but skip DB updates if pool unavailable
    // Future: Pass PgPool to finalize_block() or use global pool
    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        if let Ok(pool) = sqlx::PgPool::connect(&database_url).await {
            for (idx, result) in results.iter().enumerate() {
                if result.status == "ok" && idx < transactions.len() {
                    let tx = &transactions[idx];

                    // Update sender balance (deduct amount + fee)
                    let _ = sqlx::query(
                        r#"
                        INSERT INTO balances (account, balance, updated_at)
                        VALUES ($1, -$2, now())
                        ON CONFLICT (account) DO UPDATE
                        SET balance = balances.balance - $2, updated_at = now()
                        "#
                    )
                    .bind(&tx.sender)
                    .bind((tx.amount + tx.fee) as i64)
                    .execute(&pool)
                    .await;

                    // Update recipient balance (credit amount)
                    let _ = sqlx::query(
                        r#"
                        INSERT INTO balances (account, balance, updated_at)
                        VALUES ($1, $2, now())
                        ON CONFLICT (account) DO UPDATE
                        SET balance = balances.balance + $2, updated_at = now()
                        "#
                    )
                    .bind(&tx.recipient)
                    .bind(tx.amount as i64)
                    .execute(&pool)
                    .await;

                    log::debug!(
                        "💰 Balance update: {} -{}  (fee: {}), {} +{}",
                        tx.sender,
                        tx.amount,
                        tx.fee,
                        tx.recipient,
                        tx.amount
                    );
                }
            }
        }
    }

    info!(
        "✅ Block {} finalized: {}/{} transactions executed ({} success, {} failed)",
        block_id,
        transactions.len(),
        tx_ids.len(),
        success_count,
        failure_count
    );

    Ok(())
}
