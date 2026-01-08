-- Migration 0035: Create fraud_proofs table
-- Phase 6: Fraud proof system for verifiable batch anchoring
--
-- This table stores fraud proofs submitted by challengers who claim an aggregator
-- posted an incorrect Merkle root. If proven, the aggregator is slashed and the
-- challenger receives a reward.

CREATE TABLE IF NOT EXISTS fraud_proofs (
    -- Unique ID for this fraud proof
    id UUID PRIMARY KEY,

    -- Type of fraud being claimed
    -- Values: "InvalidMerkleRoot", "MissingTransaction", "InvalidTransaction",
    --         "DoubleInclusion", "InvalidAttestation"
    fraud_type TEXT NOT NULL,

    -- Subchain being challenged
    subchain UUID NOT NULL,

    -- Block height of anchor being challenged
    block_height BIGINT NOT NULL,

    -- Merkle root being challenged
    merkle_root BYTEA NOT NULL,

    -- Challenger's address (must have MIN_FRAUD_PROOF_STAKE)
    challenger TEXT NOT NULL,

    -- Aggregator being accused
    accused_aggregator TEXT NOT NULL,

    -- Proof data (varies by fraud type)
    -- - InvalidMerkleRoot: serialized transaction list
    -- - MissingTransaction: tx ID + Merkle proof
    -- - InvalidTransaction: invalid transaction data
    -- - DoubleInclusion: tx ID + two Merkle proofs
    -- - InvalidAttestation: attestation with invalid signature
    proof_data BYTEA NOT NULL,

    -- Additional context (JSON)
    context TEXT,

    -- When the proof was submitted
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Status of this proof
    -- Values: "Pending", "Verifying", "Proven", "Rejected", "Expired"
    status TEXT NOT NULL DEFAULT 'Pending',

    -- Result of verification (JSON, if completed)
    verification_result TEXT,

    -- When verification was completed
    verified_at TIMESTAMPTZ,

    -- Constraints
    CHECK (fraud_type IN ('InvalidMerkleRoot', 'MissingTransaction', 'InvalidTransaction',
                          'DoubleInclusion', 'InvalidAttestation')),
    CHECK (status IN ('Pending', 'Verifying', 'Proven', 'Rejected', 'Expired')),
    CHECK (verified_at IS NULL OR verified_at >= submitted_at)
);

-- Index for querying proofs by status
CREATE INDEX IF NOT EXISTS idx_fraud_proofs_status
    ON fraud_proofs(status)
    WHERE status IN ('Pending', 'Verifying');

-- Index for querying proofs by subchain and block height
CREATE INDEX IF NOT EXISTS idx_fraud_proofs_subchain_height
    ON fraud_proofs(subchain, block_height);

-- Index for querying proofs by challenger
CREATE INDEX IF NOT EXISTS idx_fraud_proofs_challenger
    ON fraud_proofs(challenger);

-- Index for querying proofs by accused aggregator
CREATE INDEX IF NOT EXISTS idx_fraud_proofs_accused
    ON fraud_proofs(accused_aggregator);

-- Index for querying proofs by merkle root (to find existing challenges)
CREATE INDEX IF NOT EXISTS idx_fraud_proofs_merkle_root
    ON fraud_proofs(merkle_root);

-- Index for querying proofs by submission time (for challenge window expiry)
CREATE INDEX IF NOT EXISTS idx_fraud_proofs_submitted_at
    ON fraud_proofs(submitted_at DESC);

-- Partial index for pending proofs that need verification
CREATE INDEX IF NOT EXISTS idx_fraud_proofs_pending_verification
    ON fraud_proofs(block_height, submitted_at)
    WHERE status = 'Pending';

COMMENT ON TABLE fraud_proofs IS
    'Phase 6: Fraud proofs for challenging incorrect anchor Merkle roots';

COMMENT ON COLUMN fraud_proofs.fraud_type IS
    'Type of fraud: InvalidMerkleRoot, MissingTransaction, InvalidTransaction, DoubleInclusion, or InvalidAttestation';

COMMENT ON COLUMN fraud_proofs.proof_data IS
    'Cryptographic proof data that demonstrates the fraud (format varies by fraud_type)';

COMMENT ON COLUMN fraud_proofs.status IS
    'Current status: Pending (awaiting verification), Verifying (in progress), Proven (fraud confirmed), Rejected (fraud disproven), Expired (challenge window closed)';

COMMENT ON COLUMN fraud_proofs.verification_result IS
    'JSON result of verification including fraud_proven, slashed_amount, reward_amount, and notes';
