-- Migration: Create node metrics and rewards tracking
-- Purpose: Track validator contributions and calculate OURO coin rewards

-- Node metrics table: tracks each validator's contributions
CREATE TABLE IF NOT EXISTS node_metrics (
    node_address TEXT PRIMARY KEY,           -- Validator public key/address
    blocks_proposed BIGINT DEFAULT 0,        -- Total blocks this node proposed
    blocks_validated BIGINT DEFAULT 0,       -- Total blocks this node validated (voted for)
    transactions_processed BIGINT DEFAULT 0, -- Total transactions in proposed blocks
    uptime_seconds BIGINT DEFAULT 0,         -- Cumulative uptime in seconds
    first_seen TIMESTAMPTZ DEFAULT NOW(),    -- When node first joined
    last_active TIMESTAMPTZ DEFAULT NOW(),   -- Last heartbeat/activity
    total_rewards BIGINT DEFAULT 0,          -- Total OURO coins earned
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for fast lookups and leaderboard queries
CREATE INDEX IF NOT EXISTS idx_node_metrics_blocks_proposed ON node_metrics(blocks_proposed DESC);
CREATE INDEX IF NOT EXISTS idx_node_metrics_total_rewards ON node_metrics(blocks_validated DESC);
CREATE INDEX IF NOT EXISTS idx_node_metrics_last_active ON node_metrics(last_active DESC);

-- Rewards history table: audit trail of all reward distributions
CREATE TABLE IF NOT EXISTS rewards_history (
    id SERIAL PRIMARY KEY,
    node_address TEXT NOT NULL,
    reward_type TEXT NOT NULL,              -- 'block_proposal', 'block_validation', 'uptime_bonus'
    amount BIGINT NOT NULL,                 -- OURO coins awarded
    block_height BIGINT,                    -- Optional: which block triggered this reward
    awarded_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB                          -- Extra info (e.g., calculation details)
);

CREATE INDEX IF NOT EXISTS idx_rewards_history_node ON rewards_history(node_address);
CREATE INDEX IF NOT EXISTS idx_rewards_history_awarded_at ON rewards_history(awarded_at DESC);

-- Reward configuration table: adjustable reward parameters
CREATE TABLE IF NOT EXISTS reward_config (
    key TEXT PRIMARY KEY,
    value BIGINT NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Default reward configuration (values in OURO coin base units)
INSERT INTO reward_config (key, value, description) VALUES
    ('block_proposal_reward', 100, 'OURO coins per block proposed'),
    ('block_validation_reward', 10, 'OURO coins per block validated'),
    ('uptime_reward_per_hour', 5, 'OURO coins per hour of uptime'),
    ('min_uptime_for_reward', 3600, 'Minimum uptime in seconds to earn rewards (1 hour)')
ON CONFLICT (key) DO NOTHING;

-- Function to automatically update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_node_metrics_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER node_metrics_update_timestamp
BEFORE UPDATE ON node_metrics
FOR EACH ROW
EXECUTE FUNCTION update_node_metrics_timestamp();

-- View for easy leaderboard queries
CREATE OR REPLACE VIEW node_leaderboard AS
SELECT
    node_address,
    blocks_proposed,
    blocks_validated,
    transactions_processed,
    total_rewards,
    uptime_seconds / 3600 as uptime_hours,
    last_active,
    CASE
        WHEN last_active > NOW() - INTERVAL '5 minutes' THEN 'online'
        WHEN last_active > NOW() - INTERVAL '1 hour' THEN 'idle'
        ELSE 'offline'
    END as status
FROM node_metrics
ORDER BY total_rewards DESC;

COMMENT ON TABLE node_metrics IS 'Tracks validator node contributions and earned rewards';
COMMENT ON TABLE rewards_history IS 'Audit trail of all reward distributions';
COMMENT ON TABLE reward_config IS 'Adjustable reward parameters for the network';
COMMENT ON VIEW node_leaderboard IS 'Top validators ranked by total rewards earned';
