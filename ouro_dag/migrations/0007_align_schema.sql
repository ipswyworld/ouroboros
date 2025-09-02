-- 0007_align_schema.sql (idempotent)

DO $$
BEGIN
  -- Rename blocks.block_id -> id if it exists
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'blocks' AND column_name = 'block_id'
  ) THEN
    EXECUTE 'ALTER TABLE blocks RENAME COLUMN block_id TO id';
    RAISE NOTICE 'Renamed blocks.block_id -> id';
  ELSE
    RAISE NOTICE 'blocks.block_id not present; skipping rename';
  END IF;

  -- Ensure tx_index table exists with expected schema
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_name = 'tx_index'
  ) THEN
    CREATE TABLE tx_index (
      tx_id UUID PRIMARY KEY,
      block_id UUID NOT NULL REFERENCES blocks(id),
      indexed_at TIMESTAMPTZ NOT NULL DEFAULT now()
    );
    RAISE NOTICE 'Created tx_index table';
  ELSE
    RAISE NOTICE 'tx_index exists; leaving intact';
  END IF;
END
$$;
