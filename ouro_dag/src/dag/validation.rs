use super::transaction::Transaction;
use std::collections::HashSet;

pub fn validate_transaction(txn: &Transaction, existing_ids: &HashSet<uuid::Uuid>) -> Result<(), String> {
    // Placeholder for validation logic
    // For now, just check if parents exist in existing_ids
    for parent_id in &txn.parents {
        if !existing_ids.contains(parent_id) {
            return Err(format!("Parent transaction {} not found", parent_id));
        }
    }
    Ok(())
}
