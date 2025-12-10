#!/bin/bash
# Ouroboros Network - Lightweight Node Setup
# Join the decentralized network in minutes (no database required!)

set -e

echo "=========================================="
echo "  🌐 Ouroboros Network - Quick Join"
echo "=========================================="
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
   echo "❌ Please do not run this script as root"
   exit 1
fi

# Detect architecture
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

if [ "$OS" != "linux" ]; then
    echo "❌ This script is for Linux. For Windows, use join_ouroboros.ps1"
    exit 1
fi

case "$ARCH" in
    x86_64)
        BINARY_NAME="ouro-node-linux-x64"
        ;;
    aarch64|arm64)
        BINARY_NAME="ouro-node-linux-arm64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        echo "   Supported: x86_64, aarch64"
        exit 1
        ;;
esac

# Create installation directory
INSTALL_DIR="$HOME/.ouroboros"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

echo "📥 Downloading Ouroboros node..."
echo "   Architecture: $ARCH"
echo ""

# Download the latest release binary
# TODO: Replace with your actual release URL
DOWNLOAD_URL="https://github.com/ipswyworld/ouroboros/releases/latest/download/$BINARY_NAME"

if command -v wget &> /dev/null; then
    wget -q --show-progress -O ouro-node "$DOWNLOAD_URL" || {
        echo "❌ Download failed. Building from source as fallback..."
        echo "   This will take longer (15-30 minutes)..."
        BUILD_FROM_SOURCE=true
    }
elif command -v curl &> /dev/null; then
    curl -L --progress-bar -o ouro-node "$DOWNLOAD_URL" || {
        echo "❌ Download failed. Building from source as fallback..."
        BUILD_FROM_SOURCE=true
    }
else
    echo "❌ Neither wget nor curl found. Installing curl..."
    sudo apt update && sudo apt install -y curl
    curl -L --progress-bar -o ouro-node "$DOWNLOAD_URL" || {
        echo "❌ Download failed. Building from source as fallback..."
        BUILD_FROM_SOURCE=true
    }
fi

# Fallback: Build from source if download fails
if [ "$BUILD_FROM_SOURCE" = true ]; then
    echo "📦 Installing dependencies..."
    sudo apt update
    sudo apt install -y build-essential pkg-config libssl-dev curl git clang libclang-dev

    # Install Rust
    if ! command -v cargo &> /dev/null; then
        echo "🦀 Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi

    echo "📥 Cloning repository..."
    cd /tmp
    rm -rf ouroboros
    git clone https://github.com/ipswyworld/ouroboros.git
    cd ouroboros/ouro_dag

    echo "🔨 Building node (this will take 15-30 minutes)..."
    cargo build --release --bin ouro-node -j 2

    cp target/release/ouro-node "$INSTALL_DIR/ouro-node"
    cd "$INSTALL_DIR"
fi

chmod +x ouro-node

echo "✅ Binary downloaded successfully"
echo ""

# Get seed node address (allow override via environment variable)
SEED_NODE="${OUROBOROS_SEED:-34.171.88.26:9001}"

echo "⚙️  Configuration:"
echo "   Storage: RocksDB (lightweight, no database needed)"
echo "   Data directory: $INSTALL_DIR/data"
echo "   Seed node: $SEED_NODE"
echo ""

# Create systemd service for auto-restart
SERVICE_FILE="$HOME/.config/systemd/user/ouroboros-node.service"
mkdir -p "$HOME/.config/systemd/user"

cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=Ouroboros Blockchain Node
After=network.target

[Service]
Type=simple
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/ouro-node join --peer $SEED_NODE --storage rocksdb --rocksdb-path $INSTALL_DIR/data --api-port 8001 --p2p-port 9001
Restart=always
RestartSec=10
StandardOutput=append:$INSTALL_DIR/node.log
StandardError=append:$INSTALL_DIR/node_error.log

[Install]
WantedBy=default.target
EOF

# Enable and start the service
systemctl --user daemon-reload
systemctl --user enable ouroboros-node.service
systemctl --user start ouroboros-node.service

echo "🚀 Starting Ouroboros node..."
sleep 3

# Check if running
if systemctl --user is-active --quiet ouroboros-node.service; then
    echo ""
    echo "=========================================="
    echo "✅ Node started successfully!"
    echo "=========================================="
    echo ""
    echo "🌐 Connected to: $SEED_NODE"
    echo "💾 Storage: RocksDB (lightweight)"
    echo "📂 Data directory: $INSTALL_DIR/data"
    echo ""
    echo "📊 Check logs:"
    echo "   tail -f $INSTALL_DIR/node.log"
    echo ""
    echo "🔍 Check node status:"
    echo "   curl http://localhost:8001/health"
    echo ""
    echo "🛠️  Manage node:"
    echo "   systemctl --user status ouroboros-node"
    echo "   systemctl --user stop ouroboros-node"
    echo "   systemctl --user restart ouroboros-node"
    echo ""
    echo "🎉 You're now part of the Ouroboros network!"
    echo "=========================================="
else
    echo ""
    echo "❌ Error: Node failed to start"
    echo "Check logs: tail -50 $INSTALL_DIR/node.log"
    echo "Check errors: tail -50 $INSTALL_DIR/node_error.log"
    exit 1
fi
