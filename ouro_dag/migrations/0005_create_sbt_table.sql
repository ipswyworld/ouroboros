CREATE TABLE IF NOT EXISTS sbt (
  sbt_id TEXT PRIMARY KEY,
  issuer TEXT NOT NULL,
  meta JSONB,
  mint_tx UUID NOT NULL,
  minted_at TIMESTAMP WITH TIME ZONE NOT NULL,
  revoked BOOLEAN DEFAULT FALSE,
  revoked_by TEXT,
  revoked_at TIMESTAMP WITH TIME ZONE
);