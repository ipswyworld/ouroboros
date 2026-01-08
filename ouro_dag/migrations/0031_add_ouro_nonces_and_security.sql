-- Migration 0031: Add nonce tracking and security constraints for Ouro Coin

-- Add nonce column to ouro_transfers
ALTER TABLE ouro_transfers ADD COLUMN IF NOT EXISTS nonce BIGINT NOT NULL DEFAULT 0;

-- Create unique index on (from_address, nonce) to prevent replay attacks
CREATE UNIQUE INDEX IF NOT EXISTS idx_ouro_transfers_from_nonce
ON ouro_transfers(from_address, nonce);

-- Account nonces table (tracks next expected nonce per address)
CREATE TABLE IF NOT EXISTS ouro_account_nonces (
    address TEXT PRIMARY KEY,
    next_nonce BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add constraints for data integrity
ALTER TABLE ouro_transfers ADD CONSTRAINT ouro_transfers_nonce_positive CHECK (nonce >= 0);

-- Index for faster nonce lookups
CREATE INDEX IF NOT EXISTS idx_ouro_account_nonces_address ON ouro_account_nonces(address);
