#!/usr/bin/env bash
set -euo pipefail

REPO="your-org/ouroboros"     # <- replace with actual GitHub repo
TAG="v0.1.0"                  # <- update per release
BIN_NAME="ouro-node"

OS="$(uname | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="arm64" ;;
esac

URL="https://github.com/${REPO}/releases/download/${TAG}/${BIN_NAME}-${OS}-${ARCH}.tar.gz"

echo "Downloading ${URL} ..."
curl -L "$URL" -o /tmp/ouro-node.tar.gz
mkdir -p /tmp/ouro-node-install
tar -xzf /tmp/ouro-node.tar.gz -C /tmp/ouro-node-install

sudo mv /tmp/ouro-node-install/$BIN_NAME /usr/local/bin/$BIN_NAME
sudo chmod +x /usr/local/bin/$BIN_NAME

echo "Installed $BIN_NAME to /usr/local/bin/$BIN_NAME"
echo "Run 'ouro-node --help' to get started."
