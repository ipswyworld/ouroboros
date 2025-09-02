CREATE TABLE IF NOT EXISTS evidence (
  id UUID PRIMARY KEY,
  validator TEXT NOT NULL,
  round BIGINT NOT NULL,
  existing_block TEXT,
  conflicting_block TEXT,
  reported_at TIMESTAMPTZ DEFAULT now()
);
