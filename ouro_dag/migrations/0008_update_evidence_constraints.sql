-- Update evidence table constraints and add index
-- This migration updates the evidence table created in 0003

-- Add NOT NULL constraints if columns exist and are nullable
ALTER TABLE evidence ALTER COLUMN existing_block SET NOT NULL;
ALTER TABLE evidence ALTER COLUMN conflicting_block SET NOT NULL;

-- Rename reported_at to observed_at if it exists
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns 
    WHERE table_name = 'evidence' AND column_name = 'reported_at'
  ) THEN
    ALTER TABLE evidence RENAME COLUMN reported_at TO observed_at;
  END IF;
END $$;

-- Add index for performance
CREATE INDEX IF NOT EXISTS idx_evidence_validator_round ON evidence (validator, round);
