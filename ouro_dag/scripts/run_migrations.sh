#!/usr/bin/env bash
set -euo pipefail

PG_HOST=${PG_HOST:-postgres}
PG_PORT=${PG_PORT:-5432}
PG_USER=${PG_USER:-ouro}

echo "[migrate] waiting for postgres at ${PG_HOST}:${PG_PORT}..."
for i in $(seq 1 60); do
  if pg_isready -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -q; then
    echo "[migrate] postgres is ready"
    break
  fi
  echo "[migrate] still waiting... ($i)"
  sleep 1
done

echo "[migrate] running SQL migrations from ./migrations"
# Run SQL files directly (idempotent runner has skip logic inside main.rs, but applying SQL here is fine)
if [ -n "${DATABASE_URL:-}" ]; then
  for f in $(ls ./migrations/*.sql | sort); do
    echo "[migrate] applying $f"
    psql "$DATABASE_URL" -f "$f"
  done
else
  echo "[migrate] DATABASE_URL not set; skipping psql-driven migrations. The binary will run programmatic migrations."
fi

echo "[migrate] launching node binary (which also runs tolerant programmatic migrations)"
exec /app/ouro_dag
