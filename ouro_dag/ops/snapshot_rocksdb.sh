#!/usr/bin/env bash
set -e
TS=$(date -u +%Y%m%dT%H%M%SZ)
SRC="/data/rocksdb"
DST="/backups/rocksdb_${TS}.tar.gz"
# stop node or ensure consistent point-in-time snapshot via rocksdb checkpoint API (preferred)
mkdir -p /tmp/rocks_checkpoint
# If you have a RocksDB checkpoint util available in your binary, call it. Otherwise stop node and tar.
tar -czf $DST -C "$SRC" .
echo "RocksDB snapshot saved to $DST"
