#!/bin/bash
sudo mkdir -p /etc/systemd/system
sudo tee /etc/systemd/system/ouro-node.service > /dev/null <<'EOF'
[Unit]
Description=Ouroboros node
After=network.target postgresql.service

[Service]
User=ouro
WorkingDirectory=/opt/ouroboros/ouro_dag
EnvironmentFile=/etc/ouro-node/ouro-node.env
ExecStart=/opt/ouroboros/ouro_dag/ops/wait-for-db-and-migrate.sh
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

sudo tee /opt/ouroboros/ouro_dag/ops/wait-for-db-and-migrate.sh > /dev/null <<'EOF'
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
  if docker exec ouro_postgres psql "$DATABASE_URL" -c '\q' >/dev/null 2>&1; then
    echo "Postgres is available"
    break
  fi
  echo "Waiting for Postgres... ($i/$RETRIES)"
  sleep $SLEEP
done

# If not reachable after retries, fail
if ! docker exec ouro_postgres psql "$DATABASE_URL" -c '\q' >/dev/null 2>&1; then
  echo "Postgres didn't become available after ${RETRIES}s"
  exit 3
fi

echo "Applying migrations in /tmp/migrations (idempotent SQL expected)"
for f in /tmp/migrations/*.sql; do
  echo "-> Applying $f"
  docker exec ouro_postgres psql "$DATABASE_URL" -f "$f"
done

echo "Migrations applied (or skipped if already present)."

/usr/local/bin/ouro-node start
EOF

sudo chmod +x /opt/ouroboros/ouro_dag/ops/wait-for-db-and-migrate.sh
sudo systemctl daemon-reload
sudo systemctl start ouro-node.service
journalctl -u ouro-node -n 50 --no-pager
