-- Migration 0029: Create Ouro Coin tables
-- OURO COIN: Native cryptocurrency with 73 million fixed supply

-- Ouro balances table
CREATE TABLE IF NOT EXISTS ouro_balances (
    address TEXT PRIMARY KEY,
    balance BIGINT NOT NULL DEFAULT 0,
    locked BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ouro_balances_balance ON ouro_balances(balance DESC);
CREATE INDEX IF NOT EXISTS idx_ouro_balances_updated ON ouro_balances(updated_at DESC);

-- Ouro transfers table
CREATE TABLE IF NOT EXISTS ouro_transfers (
    tx_id UUID PRIMARY KEY,
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    amount BIGINT NOT NULL,
    fee BIGINT NOT NULL DEFAULT 0,
    signature TEXT NOT NULL,
    public_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL DEFAULT 'pending'
);

CREATE INDEX IF NOT EXISTS idx_ouro_transfers_from ON ouro_transfers(from_address, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ouro_transfers_to ON ouro_transfers(to_address, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ouro_transfers_status ON ouro_transfers(status);
CREATE INDEX IF NOT EXISTS idx_ouro_transfers_created ON ouro_transfers(created_at DESC);

-- Ensure balances are never negative
ALTER TABLE ouro_balances ADD CONSTRAINT ouro_balances_balance_positive CHECK (balance >= 0);
ALTER TABLE ouro_balances ADD CONSTRAINT ouro_balances_locked_positive CHECK (locked >= 0);

-- Ensure transfer amounts are positive
ALTER TABLE ouro_transfers ADD CONSTRAINT ouro_transfers_amount_positive CHECK (amount > 0);
ALTER TABLE ouro_transfers ADD CONSTRAINT ouro_transfers_fee_nonnegative CHECK (fee >= 0);
