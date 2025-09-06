#!/usr/bin/env bash
# install-ourod.sh
# Minimal installer for the Ouroboros node (ouro-node).
#
# Usage:
#   VERSION=latest ./install-ourod.sh            # install latest release binary (if provided)
#   VERSION=v0.1.0 ./install-ourod.sh            # install specific tag
#   ./install-ourod.sh --from-source             # build & install from source (requires rust toolchain)
#   sudo ./install-ourod.sh                      # run with sudo to place files into /usr/local/bin and create systemd unit
#
set -euo pipefail

# -------------------------
# CONFIG (edit for your project)
# -------------------------
GITHUB_OWNER="${GITHUB_OWNER:-your-github-org-or-user}"   # replace before hosting, or export env var
GITHUB_REPO="${GITHUB_REPO:-ouro_dag}"                    # replace before hosting, or export env var
BIN_NAME="${BIN_NAME:-ouro-node}"                         # binary installed
RELAY_BIN="${RELAY_BIN:-relay}"                           # optional relay binary name (if present in release)
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
SYSTEMD_UNIT_DIR="${SYSTEMD_UNIT_DIR:-/etc/systemd/system}"
ENV_DIR="${ENV_DIR:-/etc/ouro-node}"
SERVICE_USER="${SERVICE_USER:-ouro}"
CREATE_SYSTEMD="${CREATE_SYSTEMD:-yes}"

# -------------------------
# helpers
# -------------------------
log() { printf "%s\n" "$*" >&2; }
err() { printf "ERROR: %s\n" "$*" >&2; exit 1; }

# detect OS/ARCH for release asset naming
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) ARCH="$ARCH" ;;
esac

# defaults
VERSION="${VERSION:-latest}"
FROM_SOURCE=0

# parse args
while [ $# -gt 0 ]; do
  case "$1" in
    --from-source) FROM_SOURCE=1; shift ;;
    --no-systemd) CREATE_SYSTEMD="no"; shift ;;
    --help|-h) printf "%s\n" "Usage: $0 [--from-source] [--no-systemd]"; exit 0 ;;
    *) shift ;;
  esac
done

# choose install method
if [ "$FROM_SOURCE" -eq 1 ]; then
  log "Building from source (requires Rust toolchain)..."
  if ! command -v cargo >/dev/null 2>&1; then
    err "cargo not found in PATH. Install Rust toolchain (rustup)."
  fi
  tmpd="$(mktemp -d)"
  trap 'rm -rf "$tmpd"' EXIT
  log "Cloning ${GITHUB_OWNER}/${GITHUB_REPO}..."
  git clone --depth=1 "https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}.git" "$tmpd"
  cd "$tmpd"
  log "Building release binaries..."
  cargo build --release --bins
  # copy + install
  if [ ! -f "target/release/${BIN_NAME}" ]; then
    err "Built binary not found: target/release/${BIN_NAME}"
  fi
  sudo install -m 755 "target/release/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
  if [ -f "target/release/${RELAY_BIN}" ]; then
    sudo install -m 755 "target/release/${RELAY_BIN}" "${INSTALL_DIR}/${RELAY_BIN}"
  fi
  log "Installed binaries to ${INSTALL_DIR}"
else
  # download release asset (binary) from GitHub releases
  if [ "${VERSION}" = "latest" ]; then
    # follow redirect to find tag
    redirect=$(curl -fsI "https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest" | tr -d '\r' | awk '/^location: /{print $2}' | tail -n1)
    if [ -n "$redirect" ]; then
      # redirect looks like /owner/repo/releases/tag/v0.1.0
      VERSION="$(basename "$redirect")"
      log "Resolved latest release tag: ${VERSION}"
    else
      err "Failed to resolve latest release tag. Use VERSION=... to specify."
    fi
  fi

  # expected asset name (you must publish this asset in releases):
  ASSET_NAME="${BIN_NAME}-${OS}-${ARCH}"
  ASSET_URL="https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/download/${VERSION}/${ASSET_NAME}"

  log "Downloading release binary ${ASSET_URL} ..."
  tmpf="$(mktemp)"
  trap 'rm -f "$tmpf"' EXIT
  if ! curl -fL -o "$tmpf" "$ASSET_URL"; then
    err "Failed to download ${ASSET_URL}. Make sure the asset exists in the release (or use --from-source)."
  fi
  sudo install -m 755 "$tmpf" "${INSTALL_DIR}/${BIN_NAME}"
  log "Installed ${BIN_NAME} -> ${INSTALL_DIR}/${BIN_NAME}"
  # optional relay asset
  ASSET_RELAY="${RELAY_BIN}-${OS}-${ARCH}"
  ASSET_RELAY_URL="https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/download/${VERSION}/${ASSET_RELAY}"
  if curl -fL -o /tmp/relay.asset "$ASSET_RELAY_URL" 2>/dev/null; then
    sudo install -m 755 /tmp/relay.asset "${INSTALL_DIR}/${RELAY_BIN}"
    log "Installed ${RELAY_BIN} -> ${INSTALL_DIR}/${RELAY_BIN}"
    rm -f /tmp/relay.asset
  fi
fi

# create service user if missing (best-effort)
if id -u "$SERVICE_USER" >/dev/null 2>&1; then
  log "Service user ${SERVICE_USER} exists."
else
  log "Creating system user ${SERVICE_USER}..."
  sudo useradd --system --no-create-home --shell /usr/sbin/nologin "$SERVICE_USER" || true
fi

# create env dir
if [ "$CREATE_SYSTEMD" = "yes" ]; then
  sudo mkdir -p "${ENV_DIR}"
  sudo chown "$SERVICE_USER":"$SERVICE_USER" "${ENV_DIR}" || true
  # sample env file
  cat <<'EOF' | sudo tee "${ENV_DIR}/ouro-node.env" >/dev/null
# sample env file for ouro-node (edit as needed)
API_ADDR=0.0.0.0:8000
LISTEN_ADDR=0.0.0.0:9000
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/postgres
ROCKSDB_PATH=/var/lib/ouro/kv
NODE_ID=node-1
#NODE_KEYPAIR_HEX=
PEER_ADDRS=
BOOTSTRAP_URL=
EOF
  sudo chown "$SERVICE_USER":"$SERVICE_USER" "${ENV_DIR}/ouro-node.env" || true
fi

# create systemd unit (optional)
if [ "$CREATE_SYSTEMD" = "yes" ]; then
  sudo tee "${SYSTEMD_UNIT_DIR}/ouro-node.service" >/dev/null <<EOF
[Unit]
Description=Ouroboros Node
After=network.target
Requires=network.target

[Service]
Type=simple
EnvironmentFile=${ENV_DIR}/ouro-node.env
ExecStart=${INSTALL_DIR}/${BIN_NAME} start
Restart=on-failure
User=${SERVICE_USER}
WorkingDirectory=/var/lib/ouro
RuntimeDirectory=ouro
KillMode=process

[Install]
WantedBy=multi-user.target
EOF

  sudo mkdir -p /var/lib/ouro
  sudo chown "$SERVICE_USER":"$SERVICE_USER" /var/lib/ouro || true
  log "Installed systemd unit: ${SYSTEMD_UNIT_DIR}/ouro-node.service"
  log "Enable & start with:"
  log "  sudo systemctl daemon-reload && sudo systemctl enable --now ouro-node"
fi

log "Done. Binary: ${INSTALL_DIR}/${BIN_NAME}"
