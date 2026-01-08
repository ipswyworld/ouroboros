-- 0012_provisional_and_main_anchors.sql
CREATE TABLE IF NOT EXISTS provisional_claims (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  payload jsonb,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS main_anchors (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  subchain uuid,
  block_height bigint,
  root bytea,
  posted_at timestamptz NOT NULL DEFAULT now(),
  main_tx bytea
);
