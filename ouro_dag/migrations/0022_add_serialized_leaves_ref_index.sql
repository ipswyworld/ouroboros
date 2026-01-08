-- 0022_add_serialized_leaves_ref_index.sql
CREATE INDEX IF NOT EXISTS idx_subchain_batches_serialized_ref ON subchain_batches (serialized_leaves_ref);