use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub tx_ids: Vec<Uuid>,
    pub validator_signatures: Vec<String>,
}
