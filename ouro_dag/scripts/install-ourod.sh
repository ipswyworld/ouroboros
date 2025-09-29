#!/usr/bin/env bash
set -euo pipefail

# Simple installer for ouro-node (build-from-source)
# Usage: sudo bash install-ourod.sh [--no-postgres] [--skip-rust]
# Run as root or via sudo.

REPO_DIR=${REPO_DIR:-/opt/ouroboros/ouro_dag}
BINARY_NAME=${BINARY_NAME:-ouro-node}
SERVICE_USER=${SERVICE_USER:-ouro}
SERVICE_HOME=/home/${SERVICE_USER}
ENV_FILE=/etc/ouro-node/ouro-node.env
SYSTEMD_UNIT=/etc/systemd/system/ouro-node.service

echo "==> Installer starting. REPO_DIR=${REPO_DIR}"

# 1) Create service user
if ! id -u ${SERVICE_USER} >/dev/null 2>&1; then
  echo "Creating service user ${SERVICE_USER}"
  useradd --system --create-home --shell /usr/sbin/nologin ${SERVICE_USER}
fi

# 2) Install OS packages
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y build-essential curl ca-certificates git jq \
  pkg-config libssl-dev llvm clang lld g++ libclang-dev \
  postgresql postgresql-contrib openssh-client

# 3) Optional: sccache (if available in apt)
if command -v sccache >/dev/null 2>&1; then
  echo "sccache already installed"
else
  if apt-get install -y sccache; then
    echo "installed sccache"
  fi
fi

# 4) Install rustup + toolchain if not present
if ! command -v rustc >/dev/null 2>&1; then
  echo "Installing rustup + stable toolchain"
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# Make Rust tools available for root and service user (non-login)
export PATH="$HOME/.cargo/bin:$PATH"

# 5) Clone (or update) repo
mkdir -p "$(dirname "${REPO_DIR}")"
if [ -d "${REPO_DIR}/.git" ]; then
  echo "Updating repository"
  sudo -u ${SERVICE_USER} git -C "${REPO_DIR}" pull --ff-only
else
  echo "Cloning repository to ${REPO_DIR}"
  git clone https://github.com/ipswyworld/ouroboros "${REPO_DIR}"
fi

chown -R ${SERVICE_USER}:${SERVICE_USER} "${REPO_DIR}"

# 6) Create environment directory and default env file
mkdir -p /etc/ouro-node
cat > ${ENV_FILE} <<'EOF'
# example: tune these before starting
API_ADDR=0.0.0.0:8000
LISTEN_ADDR=0.0.0.0:9000
DATABASE_URL=postgres://ouro:ouropass@127.0.0.1:5432/ourodb
NODE_ID= # optional, set to stable node id
PEER_ADDRS= # comma separated peer addresses like "34.171.55.190:9000"
BOOTSTRAP_URL= # optional
NODE_KEYPAIR_HEX= # optional 64-byte hex keypair
EOF

chmod 640 ${ENV_FILE}
chown root:root ${ENV_FILE}

# 7) Setup Postgres DB and user (if missing)
sudo -u postgres psql -v ON_ERROR_STOP=1 <<'SQL' || true
CREATE USER ouro WITH PASSWORD 'ouropass';
CREATE DATABASE ourodb OWNER ouro;
GRANT ALL PRIVILEGES ON DATABASE ourodb TO ouro;
SQL

# 8) Build the node (release)
# switch to service user to perform build for proper filesystem ownership
sudo -u ${SERVICE_USER} bash -lc "
set -e
cd ${REPO_DIR}
# source rust env if it exists for root
if [ -f /root/.cargo/env ]; then
  . /root/.cargo/env
fi

# optional: use sccache if available
if command -v sccache >/dev/null 2>&1; then
  export RUSTC_WRAPPER=\$(which sccache)
fi

export RUSTFLAGS='-C link-arg=-fuse-ld=lld'
# build release
cargo build --release
"

# 9) Install binary to /usr/local/bin
BIN_SRC="${REPO_DIR}/target/release/ouro-node"
if [ -f "${BIN_SRC}" ]; then
  cp "${BIN_SRC}" /usr/local/bin/ouro-node
  chown root:root /usr/local/bin/ouro-node
  chmod 0755 /usr/local/bin/ouro-node
else
  echo "ERROR: built binary not found at ${BIN_SRC}"
  exit 1
fi

# 10) systemd unit
cat > ${SYSTEMD_UNIT} <<EOF
[Unit]
Description=Ouroboros Node
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=${SERVICE_USER}
WorkingDirectory=${REPO_DIR}
EnvironmentFile=${ENV_FILE}
ExecStart=/usr/local/bin/ouro-node start
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now ouro-node

echo "==> Installed ouro-node. Service started (systemctl status ouro-node)."
echo "Env file: ${ENV_FILE}"
echo "Repo dir: ${REPO_DIR}"
