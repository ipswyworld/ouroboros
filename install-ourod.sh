#!/usr/bin/env bash
set -euo pipefail

# Installer for ouro-node (build-from-source)
# Default mode: use Dockerized Postgres (binds to 127.0.0.1:15432)
# Usage: sudo bash /opt/ouro/install-ourod.sh
# Optional environment variables:
#   USE_DOCKER_DB=1|0      (default: 1)
#   POSTGRES_CONTAINER_NAME (default: ouro_postgres)
#   PG_BIND_PORT (default: 15432)
#   REPO_DIR (default: /opt/ouroboros/ouro_dag)
#   REPO_URL (default: https://github.com/ipswyworld/ouroboros)
#   SERVICE_USER (default: ouro)
#   BINARY_NAME (default: ouro-node)

USE_DOCKER_DB=${USE_DOCKER_DB:-1}
POSTGRES_CONTAINER_NAME=${POSTGRES_CONTAINER_NAME:-ouro_postgres}
PG_BIND_PORT=${PG_BIND_PORT:-15432}
REPO_DIR=${REPO_DIR:-/opt/ouroboros/ouro_dag}
REPO_URL=${REPO_URL:-https://github.com/ipswyworld/ouroboros}
SERVICE_USER=${SERVICE_USER:-ouro}
BINARY_NAME=${BINARY_NAME:-ouro-node}
ENV_FILE=/etc/ouro-node/ouro-node.env
SYSTEMD_UNIT=/etc/systemd/system/ouro-node.service

echo "==> Installer starting. REPO_DIR=${REPO_DIR}"
echo "    USE_DOCKER_DB=${USE_DOCKER_DB} POSTGRES_CONTAINER_NAME=${POSTGRES_CONTAINER_NAME} PG_BIND_PORT=${PG_BIND_PORT}"

# 1) Create service user (system user, non-login)
if ! id -u "${SERVICE_USER}" >/dev/null 2>&1; then
  echo "Creating system user ${SERVICE_USER}"
  useradd --system --create-home --shell /usr/sbin/nologin "${SERVICE_USER}"
fi

# 2) Install base packages and tools
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y build-essential curl ca-certificates git jq \
  pkg-config libssl-dev llvm clang lld g++ gnupg lsb-release

# 3) Docker & Compose (only if using Docker DB)
if [ "${USE_DOCKER_DB}" = "1" ]; then
  if ! command -v docker >/dev/null 2>&1; then
    echo "Installing Docker..."
    mkdir -p /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    echo \
      "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
      $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
    apt-get update
    apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
    usermod -aG docker "${SERVICE_USER}" || true
  else
    echo "Docker present"
  fi
fi

# 4) Install rustup if missing (noninteractive)
if ! command -v rustc >/dev/null 2>&1; then
  echo "Installing rustup + stable toolchain"
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  export PATH="$HOME/.cargo/bin:$PATH"
fi
export PATH="${PATH}:/root/.cargo/bin"

# 5) Clone or update repo
mkdir -p "$(dirname "${REPO_DIR}")"
if [ -d "${REPO_DIR}/.git" ]; then
  echo "Updating repository ${REPO_DIR}"
  sudo -u "${SERVICE_USER}" git -C "${REPO_DIR}" pull --ff-only || true
else
  echo "Cloning repository ${REPO_URL} -> ${REPO_DIR}"
  git clone "${REPO_URL}" "${REPO_DIR}"
fi
chown -R "${SERVICE_USER}:${SERVICE_USER}" "${REPO_DIR}"

# 6) Postgres: Docker path or host path
if [ "${USE_DOCKER_DB}" = "1" ]; then
  # prepare docker-compose for Postgres only if not present
  mkdir -p /opt/ouro/docker
  COMPOSE_FILE=/opt/ouro/docker/docker-compose.yml
  if [ ! -f "${COMPOSE_FILE}" ]; then
    cat > "${COMPOSE_FILE}" <<'YAML'
version: "3.8"
services:
  postgres:
    image: postgres:16
    container_name: ${POSTGRES_CONTAINER_NAME}
    restart: unless-stopped
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    volumes:
      - ouro_postgres_data:/var/lib/postgresql/data
    ports:
      - "127.0.0.1:${PG_BIND_PORT}:5432"
volumes:
  ouro_postgres_data:
YAML
    # replace variables in file
    sed -i "s/${POSTGRES_CONTAINER_NAME}/${POSTGRES_CONTAINER_NAME}/g" "${COMPOSE_FILE}" || true
    sed -i "s/${PG_BIND_PORT}/${PG_BIND_PORT}/g" "${COMPOSE_FILE}" || true
  fi

  # Start Postgres container (idempotent)
  echo "Starting Postgres container (docker compose)"
  cd /opt/ouro/docker
  export POSTGRES_CONTAINER_NAME="${POSTGRES_CONTAINER_NAME}"
  export PG_BIND_PORT="${PG_BIND_PORT}"
  docker compose up -d

  # Wait for Postgres to be ready
  echo "Waiting for Postgres to become available in container ${POSTGRES_CONTAINER_NAME}..."
  for i in $(seq 1 60); do
    if docker exec -u postgres "${POSTGRES_CONTAINER_NAME}" pg_isready -q; then
      echo "Postgres is ready."
      break
    fi
    sleep 1
  done

  # Create ouro user and ourodb inside the container (idempotent)
  docker exec -i "${POSTGRES_CONTAINER_NAME}" psql -v ON_ERROR_STOP=1 -U postgres <<'SQL' || true
CREATE USER ouro WITH PASSWORD 'ouropass';
CREATE DATABASE ourodb OWNER ouro;
GRANT ALL PRIVILEGES ON DATABASE ourodb TO ouro;
SQL

else
  # Install host Postgres and create DB (old behavior)
  echo "Using host Postgres (installing via apt)"
  apt-get install -y postgresql postgresql-contrib
  sudo -u postgres psql -v ON_ERROR_STOP=1 <<'SQL' || true
CREATE USER ouro WITH PASSWORD 'ouropass';
CREATE DATABASE ourodb OWNER ouro;
GRANT ALL PRIVILEGES ON DATABASE ourodb TO ouro;
SQL
fi

# 7) Create env dir and file
mkdir -p /etc/ouro-node
cat > "${ENV_FILE}" <<EOF
# Edit values as needed
API_ADDR=0.0.0.0:8000
LISTEN_ADDR=0.0.0.0:9000
DATABASE_URL=postgres://ouro:ouropass@127.0.0.1:${PG_BIND_PORT}/ourodb
NODE_ID=
PEER_ADDRS=
NODE_KEYPAIR_HEX=
BOOTSTRAP_URL=
EOF
chmod 640 "${ENV_FILE}"
chown root:root "${ENV_FILE}"

# 8) Build the node as service user
sudo -u "${SERVICE_USER}" bash -lc "
set -e
export PATH=\$HOME/.cargo/bin:\$PATH
cd '${REPO_DIR}'
if command -v sccache >/dev/null 2>&1; then
  export RUSTC_WRAPPER=\$(which sccache)
fi
export RUSTFLAGS='-C link-arg=-fuse-ld=lld'
cargo build --release
"

# 9) Install binary
BIN_SRC=\"${REPO_DIR}/target/release/${BINARY_NAME}\"
if [ -f \"${BIN_SRC}\" ]; then
  cp \"${BIN_SRC}\" /usr/local/bin/${BINARY_NAME}
  chown root:root /usr/local/bin/${BINARY_NAME}
  chmod 0755 /usr/local/bin/${BINARY_NAME}
else
  echo \"ERROR: built binary not found at ${BIN_SRC}\"
  exit 1
fi

# 10) systemd unit (depends on docker.service if using Docker)
if [ "${USE_DOCKER_DB}" = "1" ]; then
  cat > "${SYSTEMD_UNIT}" <<UNIT
[Unit]
Description=Ouroboros Node
After=network.target docker.service
Wants=docker.service

[Service]
Type=simple
User=${SERVICE_USER}
WorkingDirectory=${REPO_DIR}
EnvironmentFile=${ENV_FILE}
ExecStart=/usr/local/bin/${BINARY_NAME} start
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
UNIT
else
  cat > "${SYSTEMD_UNIT}" <<UNIT
[Unit]
Description=Ouroboros Node
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=${SERVICE_USER}
WorkingDirectory=${REPO_DIR}
EnvironmentFile=${ENV_FILE}
ExecStart=/usr/local/bin/${BINARY_NAME} start
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
UNIT
fi

systemctl daemon-reload
systemctl enable --now ouro-node

echo "==> Installed ouro-node. Service started (systemctl status ouro-node)."
echo "Env file: ${ENV_FILE}"
echo "Repo dir: ${REPO_DIR}"
echo "If using Docker Postgres, container name: ${POSTGRES_CONTAINER_NAME}, host port: ${PG_BIND_PORT}"
