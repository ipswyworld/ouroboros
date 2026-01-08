-- 0019_create_provisional_claims.sql
CREATE TABLE IF NOT EXISTS provisional_claims (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  microchain_id text NOT NULL,
  owner text,
  amount bigint NOT NULL,
  batch_root bytea DEFAULT '\x'::bytea,
  created_at timestamptz DEFAULT now(),
  finalized boolean DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_provisional_claims_finalized ON provisional_claims (finalized);