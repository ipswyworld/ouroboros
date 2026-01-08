-- 0016_add_posted_at_to_subchain_batches.sql
ALTER TABLE subchain_batches
ADD COLUMN IF NOT EXISTS posted_at TIMESTAMPTZ;