-- 0026_add_next_retry_col.sql
ALTER TABLE IF EXISTS subchain_batches
  ADD COLUMN IF NOT EXISTS next_retry_at timestamptz DEFAULT NULL;
