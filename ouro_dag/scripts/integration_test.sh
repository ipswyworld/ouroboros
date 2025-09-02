#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# start stack
docker compose up --build -d ouro_postgres ouro_node

# wait for API
for i in {1..60}; do
  if curl -s http://127.0.0.1:8000/health | grep -q "ok"; then break; fi
  sleep 1
done

# generate signed tx (via tool)
SIGNED_JSON=$(cargo run --bin sign_tx --manifest-path ./ouro_dag/Cargo.toml --release -- <args>)

# submit
RESP=$(curl -s -X POST http://127.0.0.1:8000/tx/submit -H 'Content-Type: application/json' -d "$SIGNED_JSON")
TXID=$(echo $RESP | jq -r '.id')

# poll for inclusion
for i in {1..60}; do
  if curl -s http://127.0.0.1:8000/tx/$TXID | jq -e '.block_id' >/dev/null 2>&1; then
    echo "Included!"
    exit 0
  fi
  sleep 2
done

echo "FAIL: TX not included"
exit 2

