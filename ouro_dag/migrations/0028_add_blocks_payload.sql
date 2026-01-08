-- Migration: Add payload column to blocks table
-- Date: 2025-11-27
-- Purpose: Fix "column payload does not exist" error from consensus

-- Add payload column to store block payload data
ALTER TABLE blocks ADD COLUMN IF NOT EXISTS payload JSONB;

-- Add GIN index for efficient JSONB queries
CREATE INDEX IF NOT EXISTS idx_blocks_payload ON blocks USING gin(payload);

-- Add comment for documentation
COMMENT ON COLUMN blocks.payload IS 'Block payload data in JSONB format (added 2025-11-27)';
