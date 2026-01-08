-- 0011_microchain_and_anchor_tables.sql
ALTER TABLE microchains
  ADD COLUMN IF NOT EXISTS pubkey bytea,
  ADD COLUMN IF NOT EXISTS parent_subchain uuid;

CREATE TABLE IF NOT EXISTS subchain_batches (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subchain uuid REFERENCES subchains(id),
  batch_root bytea NOT NULL,
  aggregator text,
  leaf_count bigint,
  serialized_leaves_ref text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_subchain_batches_subchain ON subchain_batches(subchain);
