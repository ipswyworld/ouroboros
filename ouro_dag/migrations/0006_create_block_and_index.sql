-- 0006_create_block_and_index.sql
CREATE TABLE IF NOT EXISTS blocks (
  id UUID PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  payload JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS tx_index (
  tx_id UUID PRIMARY KEY,
  block_id UUID NOT NULL REFERENCES blocks(id),
  indexed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
