-- Migration 0037: Add balance underflow protection
-- SECURITY FIX: Prevent negative balances and money creation exploits
--
-- This migration adds database-level constraints to prevent balance underflows
-- that could allow money creation or negative balances during slashing operations.

-- Ensure balances table has non-negative balance constraint
ALTER TABLE balances
  ADD CONSTRAINT IF NOT EXISTS balances_balance_nonnegative CHECK (balance >= 0);

-- Ensure amount column exists and is non-negative in balances table
-- (Note: Some tables use 'amount' instead of 'balance')
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'balances' AND column_name = 'amount'
  ) THEN
    ALTER TABLE balances
      ADD CONSTRAINT IF NOT EXISTS balances_amount_nonnegative CHECK (amount >= 0);
  END IF;
END $$;

-- Create index for efficient balance lookups
CREATE INDEX IF NOT EXISTS idx_balances_balance ON balances(balance DESC);

-- Log completion
DO $$
BEGIN
  RAISE NOTICE '✅ Balance underflow protection enabled - all balances must be >= 0';
END $$;
