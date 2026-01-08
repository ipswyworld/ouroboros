-- 0020_create_subchains_and_microchains.sql
CREATE TABLE IF NOT EXISTS subchains (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name text,
  created_at timestamptz DEFAULT now(),
  status text DEFAULT 'active',
  max_micro int DEFAULT 1000,
  current_micro int DEFAULT 0
);

CREATE TABLE IF NOT EXISTS microchains (
  id uuid PRIMARY KEY,
  owner text,
  pubkey bytea,
  parent_subchain uuid,
  created_at timestamptz DEFAULT now(),
  status text DEFAULT 'active'
);

CREATE INDEX IF NOT EXISTS idx_microchains_parent ON microchains (parent_subchain);