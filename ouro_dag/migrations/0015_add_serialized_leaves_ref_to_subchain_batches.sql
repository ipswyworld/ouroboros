-- 0015_add_serialized_leaves_ref_to_subchain_batches.sql
ALTER TABLE subchain_batches
ADD COLUMN IF NOT EXISTS serialized_leaves_ref TEXT;