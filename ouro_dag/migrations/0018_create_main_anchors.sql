-- 0018_create_main_anchors.sql
CREATE TABLE IF NOT EXISTS main_anchors (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subchain uuid,
  block_height bigint,
  root bytea,
  txid bytea,
  posted_at timestamptz DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_main_anchors_subchain ON main_anchors (subchain);