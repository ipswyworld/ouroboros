-- Migration: Create subchain registry and rent system
-- Description: Manages subchain lifecycle, rent payments, and discovery

CREATE TABLE IF NOT EXISTS subchain_registry (
    -- Identity
    id UUID PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    owner_address VARCHAR(256) NOT NULL,

    -- Rent economics
    deposit_balance BIGINT NOT NULL DEFAULT 0,
    last_rent_block BIGINT NOT NULL DEFAULT 0,
    total_rent_paid BIGINT NOT NULL DEFAULT 0,

    -- Lifecycle state
    state TEXT NOT NULL DEFAULT 'active' CHECK (state IN ('active', 'grace_period', 'terminated')),
    grace_period_start TIMESTAMPTZ,
    grace_period_start_block BIGINT,

    -- Configuration
    anchor_frequency BIGINT NOT NULL DEFAULT 100,
    rpc_endpoint TEXT,

    -- Metrics
    total_blocks_served BIGINT NOT NULL DEFAULT 0,

    -- Timestamps
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX idx_subchain_registry_owner ON subchain_registry(owner_address);
CREATE INDEX idx_subchain_registry_state ON subchain_registry(state);
CREATE INDEX idx_subchain_registry_name ON subchain_registry(name);

-- Index for rent collection queries
CREATE INDEX idx_subchain_registry_rent_due ON subchain_registry(last_rent_block)
WHERE state IN ('active', 'grace_period');

-- Comments
COMMENT ON TABLE subchain_registry IS 'Registry of all subchains with rent and lifecycle management';
COMMENT ON COLUMN subchain_registry.deposit_balance IS 'Current rent deposit balance in OURO smallest units';
COMMENT ON COLUMN subchain_registry.last_rent_block IS 'Block height when rent was last charged';
COMMENT ON COLUMN subchain_registry.state IS 'Lifecycle state: active, grace_period, terminated';
COMMENT ON COLUMN subchain_registry.grace_period_start_block IS 'Block when grace period started (1440 blocks = ~2 hours)';
COMMENT ON COLUMN subchain_registry.anchor_frequency IS 'How often subchain anchors to mainchain (in blocks)';
