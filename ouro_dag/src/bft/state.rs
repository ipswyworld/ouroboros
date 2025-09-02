use std::collections::HashMap;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct Equivocation {
    pub validator: String,
    pub round: u64,
    pub existing: String,
    pub conflicting: String,
}

pub struct BFTState {
    // key: (validator_id, round), value: block_hash
    pub seen_signatures: HashMap<(String, u64), String>,
    pub db_pool: PgPool,
}

impl BFTState {
    pub fn new(db_pool: PgPool) -> Self {
        BFTState {
            seen_signatures: HashMap::new(),
            db_pool,
        }
    }

    pub async fn record_signature(&mut self, validator: &str, round: u64, block_hash: &str) -> Result<(), Equivocation> {
        let key = (validator.to_string(), round);
        if let Some(existing) = self.seen_signatures.get(&key) {
            if existing != block_hash {
                let equivocation = Equivocation {
                    validator: validator.to_string(),
                    round,
                    existing: existing.clone(),
                    conflicting: block_hash.to_string(),
                };
                self.persist_evidence(&equivocation).await;
                return Err(equivocation);
            } else {
                return Ok(());
            }
        }
        self.seen_signatures.insert(key, block_hash.to_string());
        Ok(())
    }

    async fn persist_evidence(&self, evidence: &Equivocation) {
        let _ = sqlx::query(
            r#"
            INSERT INTO evidence (id, validator, round, existing_block, conflicting_block)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&evidence.validator)
        .bind(evidence.round as i64)
        .bind(&evidence.existing)
        .bind(&evidence.conflicting)
        .execute(&self.db_pool)
        .await;
    }
}
