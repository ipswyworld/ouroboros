# Database Migration Order

**Last Updated:** 2025-11-24
**Status:** Consolidated and cleaned

## Migration Sequence

| # | File | Description | Status |
|---|------|-------------|--------|
| 0001 | create_chain_schema.sql | Initial schema (transactions, blocks, tx_index) | ✅ Core |
| 0002 | add_idempotency_and_nonce.sql | Idempotency keys, nonce tracking | ✅ Core |
| 0003 | create_evidence_table.sql | Evidence table for slashing | ✅ Core |
| 0004 | create_mempool_table.sql | Persistent mempool entries | ✅ Core |
| 0005 | create_sbt_table.sql | Soulbound tokens | ✅ VM |
| 0006 | create_block_and_index.sql | Block metadata and indexes | ✅ Core |
| 0007 | align_schema.sql | Schema alignment fixes | ✅ Core |
| 0008 | update_evidence_constraints.sql | Add NOT NULL, index to evidence | ✅ Core |
| 0009 | create_bft_meta.sql | BFT metadata tables | ✅ Consensus |
| 0010 | create_subchain_and_micro_tables.sql | Subchain/microchain tables | ✅ Scaling |
| 0011 | microchain_and_anchor_tables.sql | Microchain anchors | ✅ Scaling |
| 0012 | provisional_and_main_anchors.sql | Anchor types | ✅ Scaling |
| 0013 | escrow_balances.sql | Escrow system | ✅ Escrow |
| 0014 | provisional_balances.sql | Provisional balance tracking | ✅ Scaling |
| 0015 | add_serialized_leaves_ref_to_subchain_batches.sql | Batch serialization | ✅ Scaling |
| 0016 | add_posted_at_to_subchain_batches.sql | Batch timestamps | ✅ Scaling |
| 0017 | add_verified_and_main_tx_to_subchain_batches.sql | Batch verification | ✅ Scaling |
| 0018 | create_main_anchors.sql | Main anchor table | ✅ Scaling |
| 0019 | create_provisional_claims.sql | Provisional claim tracking | ✅ Scaling |
| 0020 | create_subchains_and_microchains.sql | Chain registries | ✅ Scaling |
| 0021 | create_provisional_balances.sql | Provisional balances | ✅ Scaling |
| 0022 | add_serialized_leaves_ref_index.sql | Index for batch lookups | ✅ Scaling |
| 0023 | create_balances_table.sql | Main balance table | ✅ Core |
| 0024 | add_batch_root_to_provisional_claims.sql | Batch root tracking | ✅ Scaling |
| 0026 | poster_idempotency_consolidated.sql | Poster retry/idempotency | ✅ Scaling |
| 0027 | add_next_retry_col.sql | Batch retry scheduling | ✅ Scaling |

## Table Dependencies

```
transactions → tx_index
transactions → blocks
transactions → mempool_entries
evidence → (validators)
bft_meta → blocks
subchain_batches → main_anchors
microchains → subchains
escrows → balances
provisional_claims → subchain_batches
```

## Testing Migrations

```bash
# Test on clean database
psql postgres://ouro:ouro_pass@localhost:15432/ouro_test -c "DROP DATABASE IF EXISTS ouro_test;"
psql postgres://ouro:ouro_pass@localhost:15432/postgres -c "CREATE DATABASE ouro_test;"

# Run migrations
for f in migrations/*.sql; do
  echo "Running $f..."
  psql postgres://ouro:ouro_pass@localhost:15432/ouro_test -f "$f"
done

# Verify schema
psql postgres://ouro:ouro_pass@localhost:15432/ouro_test -c "\dt"
psql postgres://ouro:ouro_pass@localhost:15432/ouro_test -c "\di"
```

## Issues Fixed

✅ **Fixed:** Duplicate 0026 files (renamed to 0027)
✅ **Fixed:** Duplicate evidence table creation (converted to ALTER TABLE)
✅ **Verified:** All migrations are idempotent (IF NOT EXISTS)
✅ **Verified:** No missing sequence numbers (0025 intentionally skipped in cleanup)

## Notes

- Migration 0025 was split into multiple files and consolidated into 0026
- All migrations use `IF NOT EXISTS` / `IF EXISTS` for safety
- Can be run multiple times without errors
- Schema supports full Ouroboros architecture (Main → Sub → Microchains)
