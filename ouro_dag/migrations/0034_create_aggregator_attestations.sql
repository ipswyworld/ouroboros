-- Migration 0034: Create aggregator_attestations table
-- Phase 6: Aggregator attestations for verifiable anchor posting
--
-- This table stores cryptographic attestations from aggregators that prove
-- they correctly computed the Merkle root for a batch. These attestations
-- enable fraud proofs and slashing for malicious aggregators.

CREATE TABLE IF NOT EXISTS aggregator_attestations (
    -- Subchain UUID this attestation is for
    subchain UUID NOT NULL,

    -- Block height being anchored
    block_height BIGINT NOT NULL,

    -- Merkle root of the batch (32 bytes)
    merkle_root BYTEA NOT NULL,

    -- Number of transactions in the batch
    tx_count BIGINT NOT NULL CHECK (tx_count >= 0),

    -- Total batch size in bytes
    batch_size_bytes BIGINT NOT NULL CHECK (batch_size_bytes >= 0),

    -- Aggregator's Ed25519 public key (32 bytes)
    aggregator_pubkey BYTEA NOT NULL CHECK (length(aggregator_pubkey) = 32),

    -- Ed25519 signature over attestation data (64 bytes)
    signature BYTEA NOT NULL CHECK (length(signature) = 64),

    -- Timestamp when attestation was created
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Optional: Hash of serialized transaction list for fraud proof verification
    tx_list_hash BYTEA,

    -- Primary key: unique per subchain, height, and root
    PRIMARY KEY (subchain, block_height, merkle_root)
);

-- Index for querying attestations by subchain and block height
CREATE INDEX IF NOT EXISTS idx_attestations_subchain_height
    ON aggregator_attestations(subchain, block_height);

-- Index for querying attestations by aggregator
CREATE INDEX IF NOT EXISTS idx_attestations_aggregator
    ON aggregator_attestations(aggregator_pubkey);

-- Index for querying recent attestations
CREATE INDEX IF NOT EXISTS idx_attestations_created_at
    ON aggregator_attestations(created_at DESC);

COMMENT ON TABLE aggregator_attestations IS
    'Phase 6: Cryptographic attestations from aggregators proving correct Merkle root computation';

COMMENT ON COLUMN aggregator_attestations.merkle_root IS
    'Merkle root hash that the aggregator claims is correct for this batch';

COMMENT ON COLUMN aggregator_attestations.signature IS
    'Ed25519 signature over (subchain || block_height || merkle_root || tx_count || batch_size_bytes || created_at || tx_list_hash)';

COMMENT ON COLUMN aggregator_attestations.tx_list_hash IS
    'Optional hash of the full transaction list, used for fraud proof verification';
