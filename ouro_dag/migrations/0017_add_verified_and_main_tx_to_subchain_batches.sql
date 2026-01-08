-- 0017_add_verified_and_main_tx_to_subchain_batches.sql
ALTER TABLE subchain_batches
  ADD COLUMN IF NOT EXISTS verified boolean DEFAULT false,
  ADD COLUMN IF NOT EXISTS posted_at timestamptz,
  ADD COLUMN IF NOT EXISTS main_tx bytea,
  ADD COLUMN IF NOT EXISTS serialized_leaves_ref text;
-- create useful index for lookup
CREATE INDEX IF NOT EXISTS idx_subchain_batches_verified_posted ON subchain_batches (verified, posted_at);