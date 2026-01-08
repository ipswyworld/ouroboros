-- Migration: Create key_rotations table for BFT validator key rotation
-- Description: Allows validators to rotate their signing keys without downtime
-- Security: Supports transition periods where both old and new keys are valid

CREATE TABLE IF NOT EXISTS key_rotations (
    id SERIAL PRIMARY KEY,

    -- Validator identity
    validator_id TEXT NOT NULL,

    -- Key rotation data
    old_public_key TEXT NOT NULL,
    new_public_key TEXT NOT NULL,

    -- Proof of authority: new key signed by old key
    signature TEXT NOT NULL,

    -- Timing
    announced_at TIMESTAMPTZ NOT NULL,
    transition_ends_at TIMESTAMPTZ NOT NULL,

    -- Status: Pending, InTransition, Completed, Revoked
    status TEXT NOT NULL CHECK (status IN ('Pending', 'InTransition', 'Completed', 'Revoked')),

    -- Audit trail
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for querying active rotations by validator
CREATE INDEX idx_key_rotations_validator ON key_rotations(validator_id, announced_at DESC);

-- Index for processing rotations by status and time
CREATE INDEX idx_key_rotations_status ON key_rotations(status, transition_ends_at);

-- Ensure only one active rotation per validator at a time
CREATE UNIQUE INDEX idx_key_rotations_unique_active
ON key_rotations(validator_id)
WHERE status IN ('Pending', 'InTransition');

COMMENT ON TABLE key_rotations IS 'BFT validator key rotation events with transition periods';
COMMENT ON COLUMN key_rotations.signature IS 'New public key signed by old private key (proof of authority)';
COMMENT ON COLUMN key_rotations.transition_ends_at IS 'When old key expires (24 hours after announcement)';
