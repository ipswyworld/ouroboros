-- 0021_create_provisional_balances.sql
CREATE TABLE IF NOT EXISTS provisional_balances (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  microchain_id uuid,
  account text,
  amount bigint,
  created_at timestamptz DEFAULT now()
);