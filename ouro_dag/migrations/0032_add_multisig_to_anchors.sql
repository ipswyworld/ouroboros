-- 0032_add_multisig_to_anchors.sql
-- Add multi-signature support to main_anchors table

-- Add multi-sig columns to main_anchors
ALTER TABLE main_anchors
  ADD COLUMN IF NOT EXISTS multisig_data jsonb,
  ADD COLUMN IF NOT EXISTS threshold int,
  ADD COLUMN IF NOT EXISTS signature_count int DEFAULT 0;

-- Update existing rows to have default values
UPDATE main_anchors
SET
  threshold = 1,
  signature_count = 1
WHERE threshold IS NULL;

-- Add check constraint to ensure signature_count >= threshold
ALTER TABLE main_anchors
  ADD CONSTRAINT check_signature_threshold
  CHECK (signature_count >= threshold);

-- Create partial signature tracking table
CREATE TABLE IF NOT EXISTS anchor_partial_signatures (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  anchor_root bytea NOT NULL,
  validator_id text NOT NULL,
  signature bytea NOT NULL,
  created_at timestamptz DEFAULT now(),
  UNIQUE(anchor_root, validator_id)
);

-- Index for fast lookup by anchor root
CREATE INDEX IF NOT EXISTS idx_partial_sigs_root
  ON anchor_partial_signatures(anchor_root);

-- Create multi-sig validator keys table
CREATE TABLE IF NOT EXISTS multisig_validator_keys (
  validator_id text PRIMARY KEY,
  pubkey bytea NOT NULL,
  added_at timestamptz DEFAULT now(),
  is_active boolean DEFAULT true
);

-- Index for active validators
CREATE INDEX IF NOT EXISTS idx_multisig_validators_active
  ON multisig_validator_keys(is_active)
  WHERE is_active = true;

COMMENT ON TABLE anchor_partial_signatures IS
  'Stores partial signatures from validators for multi-sig anchor coordination';

COMMENT ON TABLE multisig_validator_keys IS
  'Stores public keys of validators authorized to sign anchors (M-of-N multi-sig)';

COMMENT ON COLUMN main_anchors.multisig_data IS
  'JSON blob containing all partial signatures and metadata for multi-sig verification';

COMMENT ON COLUMN main_anchors.threshold IS
  'Minimum number of signatures required (M in M-of-N multi-sig)';

COMMENT ON COLUMN main_anchors.signature_count IS
  'Actual number of valid signatures collected';
