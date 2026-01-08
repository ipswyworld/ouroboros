-- migrations/0010_create_subchain_and_micro_tables.sql
CREATE TABLE IF NOT EXISTS subchains (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  status text NOT NULL DEFAULT 'active',
  max_micro int NOT NULL DEFAULT 1000,
  current_micro int NOT NULL DEFAULT 0,
  config jsonb
);

CREATE TABLE IF NOT EXISTS microchains (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  owner text,
  parent_subchain uuid REFERENCES subchains(id),
  pubkey bytea,
  created_at timestamptz NOT NULL DEFAULT now(),
  status text NOT NULL DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS subchain_batches (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subchain uuid REFERENCES subchains(id),
  batch_root bytea NOT NULL,
  aggregator text,
  leaf_count bigint,
  created_at timestamptz NOT NULL DEFAULT now()
);
