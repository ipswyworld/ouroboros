-- blocks table
CREATE TABLE IF NOT EXISTS blocks (
  block_id UUID PRIMARY KEY,
  block_height BIGINT NOT NULL,
  parent_ids UUID[] NOT NULL,
  merkle_root TEXT NOT NULL,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
  tx_count INT NOT NULL,
  block_bytes BYTEA NOT NULL,
  signer TEXT,
  signature TEXT,
  created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_blocks_height ON blocks(block_height);

-- transactions table (persisted)
CREATE TABLE IF NOT EXISTS transactions (
  tx_id UUID PRIMARY KEY,
  tx_hash TEXT UNIQUE,
  sender TEXT,
  recipient TEXT,
  payload JSONB,            -- full transaction payload
  status TEXT DEFAULT 'pending', -- pending, included, failed
  included_in_block UUID NULL,
  created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_tx_sender ON transactions(sender);
CREATE INDEX idx_tx_status ON transactions(status);

-- tx_index table (for quick lookup)
CREATE TABLE IF NOT EXISTS tx_index (
  tx_hash TEXT PRIMARY KEY,
  tx_id UUID NOT NULL,
  block_id UUID NULL,
  created_at TIMESTAMPTZ DEFAULT now()
);

-- mempool pointers / persistent mempool entries
CREATE TABLE IF NOT EXISTS mempool_entries (
  tx_id UUID PRIMARY KEY,
  tx_hash TEXT,
  payload JSONB,
  received_at TIMESTAMPTZ DEFAULT now()
);
