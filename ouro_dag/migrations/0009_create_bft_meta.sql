-- migrations/0009_create_bft_meta.sql
CREATE TABLE IF NOT EXISTS bft_meta (
  key text PRIMARY KEY,
  value jsonb NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- seed default
INSERT INTO bft_meta (key, value) VALUES ('current_view', '"0"') ON CONFLICT DO NOTHING;
