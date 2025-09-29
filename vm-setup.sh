#!/usr/bin/env bash
set -euo pipefail

echo "--- 1. Installing Rust for user ouro ---"
sudo -u ouro bash -c 'command -v rustc &>/dev/null || (curl https://sh.rustup.rs -sSf | sh -s -- -y)'

echo "--- 2. Creating environment file ---"
sudo mkdir -p /etc/ouro-node
sudo tee /etc/ouro-node/ouro-node.env > /dev/null <<'EOT'
API_ADDR=0.0.0.0:8000
LISTEN_ADDR=0.0.0.0:9000
DATABASE_URL=postgres://ouro:ouropass@127.0.0.1:15432/ourodb
NODE_ID=bootstrap-1
PEER_ADDRS=
NODE_KEYPAIR_HEX=
BOOTSTRAP_URL=
EOT
sudo chmod 640 /etc/ouro-node/ouro-node.env
sudo chown root:ouro /etc/ouro-node/ouro-node.env

echo "--- 3. Building ouro-node application (this will take a few minutes) ---"
sudo -u ouro bash -c 'source "$HOME/.cargo/env"; cd /opt/ouroboros/ouro_dag && cargo build --release'

echo "--- 4. Installing application and systemd service ---"
sudo cp /opt/ouroboros/ouro_dag/target/release/ouro-node /usr/local/bin/
sudo cp /opt/ouroboros/ouro_dag/ouro-node.service /etc/systemd/system/

echo "--- 5. Starting ouro-node service ---"
sudo systemctl daemon-reload
sudo systemctl enable --now ouro-node

echo "--- 6. Checking service status (waiting 5 seconds) ---"
sleep 5
sudo systemctl status ouro-node --no-pager

echo "--- 7. Checking logs ---"
journalctl -u ouro-node -n 20 --no-pager

echo "--- 8. Checking health endpoint ---"
curl -sS http://127.0.0.1:8000/health || echo "Health endpoint not responding yet."