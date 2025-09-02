ALTER TABLE transactions ADD COLUMN idempotency_key TEXT NULL;
ALTER TABLE transactions ADD COLUMN nonce BIGINT NULL;
CREATE UNIQUE INDEX idx_idempotency ON transactions(idempotency_key) WHERE idempotency_key IS NOT NULL;
