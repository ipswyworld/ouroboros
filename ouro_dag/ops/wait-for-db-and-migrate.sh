#!/usr/bin/env bash
set -euo pipefail

# Usage: DATABASE_URL=postgres://user:pass@host:port/db ./ops/wait-for-db-and-migrate.sh

if [ -z "${DATABASE_URL:-}" ]; then
  echo "DATABASE_URL environment variable is required"
  exit 2
fi

# Timeout/retries (seconds)
RETRIES=60
SLEEP=1

echo "Waiting for DB to become available: $DATABASE_URL"
for i in $(seq 1 $RETRIES); do
  if psql "$DATABASE_URL" -c '\q' >/dev/null 2>&1; then
    echo "Postgres is available"
    break
  fi
  echo "Waiting for Postgres... ($i/$RETRIES)"
  sleep $SLEEP
done

# If not reachable after retries, fail
if ! psql "$DATABASE_URL" -c '\q' >/dev/null 2>&1; then
  echo "Postgres didn't become available after ${RETRIES}s"
  exit 3
fi

echo "Applying migrations in ./migrations (idempotent SQL expected)"
for f in ./migrations/*.sql; do
  echo "-> Applying $f"
  psql "$DATABASE_URL" -f "$f"
done

echo "Migrations applied (or skipped if already present)."
