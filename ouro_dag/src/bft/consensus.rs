use super::block::Block;
use uuid::Uuid;
use chrono::Utc;

pub struct BFTNode {
    pub name: String,
    pub private_key: String, // Simulated key
}

impl BFTNode {
    pub fn sign_block(&self, block_id: &Uuid) -> String {
        format!("sig:{}:{}", self.name, block_id)
    }
}

pub fn finalize_block(tx_ids: Vec<Uuid>, validators: &[BFTNode]) -> Block {
    let id = Uuid::new_v4();
    let timestamp = Utc::now();

    let signatures = validators
        .iter()
        .map(|v| v.sign_block(&id))
        .collect::<Vec<_>>();

    Block {
        id,
        timestamp,
        tx_ids,
        validator_signatures: signatures,
    }
}
