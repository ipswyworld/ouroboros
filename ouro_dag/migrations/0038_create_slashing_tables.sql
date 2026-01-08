-- Migration 0038: Create slashing tables
-- Tracks validator stakes and slashing events

-- Validator stakes table
CREATE TABLE IF NOT EXISTS validator_stakes (
    validator_id TEXT PRIMARY KEY,
    stake BIGINT NOT NULL DEFAULT 0,
    slashed_amount BIGINT NOT NULL DEFAULT 0,
    last_slashed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure stakes are never negative
    CONSTRAINT validator_stakes_stake_positive CHECK (stake >= 0),
    CONSTRAINT validator_stakes_slashed_positive CHECK (slashed_amount >= 0)
);

-- Slashing events table
CREATE TABLE IF NOT EXISTS slashing_events (
    id SERIAL PRIMARY KEY,
    validator_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    severity TEXT NOT NULL,
    stake_before BIGINT NOT NULL,
    slashed_amount BIGINT NOT NULL,
    stake_after BIGINT NOT NULL,
    slashed_at TIMESTAMPTZ NOT NULL,
    evidence TEXT NOT NULL,

    -- Foreign key to validator stakes
    CONSTRAINT fk_validator FOREIGN KEY (validator_id)
        REFERENCES validator_stakes(validator_id) ON DELETE CASCADE
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_validator_stakes_stake ON validator_stakes(stake DESC);
CREATE INDEX IF NOT EXISTS idx_validator_stakes_updated ON validator_stakes(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_slashing_events_validator ON slashing_events(validator_id, slashed_at DESC);
CREATE INDEX IF NOT EXISTS idx_slashing_events_time ON slashing_events(slashed_at DESC);
CREATE INDEX IF NOT EXISTS idx_slashing_events_reason ON slashing_events(reason);

-- Initial validator stakes (populate from existing data if needed)
-- This can be populated by the application during validator registration

COMMENT ON TABLE validator_stakes IS 'Tracks validator stake amounts and slashing history';
COMMENT ON TABLE slashing_events IS 'Records all slashing events with evidence';
COMMENT ON COLUMN validator_stakes.stake IS 'Current stake amount in smallest units';
COMMENT ON COLUMN validator_stakes.slashed_amount IS 'Total amount slashed over lifetime';
COMMENT ON COLUMN slashing_events.evidence IS 'Evidence of violation (block ID, view, etc.)';
