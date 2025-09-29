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

echo "[migrate] keeping container alive for debugging"
tail -f /dev/null