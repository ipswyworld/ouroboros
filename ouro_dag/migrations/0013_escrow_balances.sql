-- 0013_escrow_balances.sql
CREATE TABLE IF NOT EXISTS balances (
  account text PRIMARY KEY,
  balance bigint NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS escrows (
  escrow_id uuid PRIMARY KEY,
  from_account text,
  to_microchain uuid,
  amount bigint,
  nonce bigint,
  expiry timestamptz,
  status text,
  created_at timestamptz DEFAULT now(),
  finalized_at timestamptz,
  refunded_at timestamptz
);
