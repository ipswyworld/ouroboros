CREATE TABLE IF NOT EXISTS mempool (
  tx_id UUID PRIMARY KEY,
  transaction_data BYTEA NOT NULL,
  received_at TIMESTAMPTZ DEFAULT now()
);
