-- 0024_add_batch_root_to_provisional_claims.sql
ALTER TABLE provisional_claims
  ADD COLUMN IF NOT EXISTS batch_root bytea DEFAULT '\x'::bytea;