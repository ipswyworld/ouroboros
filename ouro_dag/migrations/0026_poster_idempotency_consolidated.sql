-- migrations/0026_poster_idempotency_consolidated.sql
-- Consolidates all 0025 migrations into one clean file
-- Adds idempotency support for poster with retry/backoff mechanism

-- Add main_tx column to track blockchain transaction hash
ALTER TABLE main_anchors ADD COLUMN IF NOT EXISTS main_tx BYTEA;

-- Add attempt tracking columns to subchain_batches
ALTER TABLE subchain_batches ADD COLUMN IF NOT EXISTS attempts INTEGER DEFAULT 0;
ALTER TABLE subchain_batches ADD COLUMN IF NOT EXISTS last_attempt TIMESTAMPTZ;
ALTER TABLE subchain_batches ADD COLUMN IF NOT EXISTS last_error TEXT;

-- Create unique indexes for idempotency (IF NOT EXISTS for safety)
CREATE UNIQUE INDEX IF NOT EXISTS idx_main_anchors_root_unique ON main_anchors (root);
CREATE UNIQUE INDEX IF NOT EXISTS idx_subchain_batches_batch_root_unique ON subchain_batches (batch_root);

-- Add helpful indexes for poster queries
CREATE INDEX IF NOT EXISTS idx_subchain_batches_attempts ON subchain_batches (attempts);
CREATE INDEX IF NOT EXISTS idx_subchain_batches_verified_posted ON subchain_batches (verified, posted_at) WHERE posted_at IS NULL;
