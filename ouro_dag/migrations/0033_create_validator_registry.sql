-- 0033_create_validator_registry.sql
-- Permissionless validator registration system
-- Allows anyone to become a validator by staking OURO tokens

-- Validator registry table
CREATE TABLE IF NOT EXISTS validator_registry (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Validator's Ouro address (must hold staked OURO)
    address text NOT NULL UNIQUE,
    -- Ed25519 public key for BFT consensus (hex encoded)
    bft_pubkey text NOT NULL UNIQUE,
    -- Network endpoint for P2P (IP:port or .onion:port)
    network_endpoint text NOT NULL,
    -- BFT consensus port
    bft_port int NOT NULL,
    -- Amount staked in OURO microunits (1 OURO = 1,000,000 microunits)
    stake_amount bigint NOT NULL CHECK (stake_amount > 0),
    -- Current status: pending, active, unbonding, slashed, exited
    status text NOT NULL DEFAULT 'pending',
    -- Reputation score (0-100)
    reputation int NOT NULL DEFAULT 50 CHECK (reputation >= 0 AND reputation <= 100),
    -- Performance metrics
    blocks_proposed bigint NOT NULL DEFAULT 0,
    blocks_signed bigint NOT NULL DEFAULT 0,
    missed_proposals bigint NOT NULL DEFAULT 0,
    -- Slashing tracking
    slashed_amount bigint NOT NULL DEFAULT 0,
    -- Timestamps
    registered_at timestamptz NOT NULL DEFAULT now(),
    activated_at timestamptz,
    exit_requested_at timestamptz,
    unbonding_complete_at timestamptz,

    -- Constraints
    CONSTRAINT valid_status CHECK (status IN ('pending', 'active', 'unbonding', 'slashed', 'exited'))
);

-- Slashing events table
CREATE TABLE IF NOT EXISTS slashing_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    validator_id uuid NOT NULL REFERENCES validator_registry(id) ON DELETE CASCADE,
    -- Reason: double_sign, downtime, invalid_block, byzantine
    reason text NOT NULL,
    -- Amount slashed in OURO microunits
    amount_slashed bigint NOT NULL,
    -- Evidence (JSON blob with proof of misbehavior)
    evidence jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Indexes for fast queries
CREATE INDEX IF NOT EXISTS idx_validator_status
    ON validator_registry(status);

CREATE INDEX IF NOT EXISTS idx_validator_stake
    ON validator_registry(stake_amount DESC)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS idx_validator_reputation
    ON validator_registry(reputation DESC)
    WHERE status = 'active';

CREATE INDEX IF NOT EXISTS idx_slashing_validator
    ON slashing_events(validator_id, created_at DESC);

-- Comments
COMMENT ON TABLE validator_registry IS
    'Permissionless validator registry - anyone can register by staking OURO';

COMMENT ON COLUMN validator_registry.stake_amount IS
    'Stake in OURO microunits (minimum 200,000,000 = 200 OURO)';

COMMENT ON COLUMN validator_registry.status IS
    'pending: awaiting activation, active: in consensus, unbonding: exit requested, slashed: penalized, exited: complete';

COMMENT ON TABLE slashing_events IS
    'Records of validator slashing for Byzantine behavior or downtime';
