-- 0023_create_balances_table.sql
CREATE TABLE IF NOT EXISTS balances (
  account text PRIMARY KEY,
  balance bigint DEFAULT 0,
  updated_at timestamptz DEFAULT now()
);