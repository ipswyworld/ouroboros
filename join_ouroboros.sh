#!/bin/bash
set -e

NODE_DIR="$HOME/.ouroboros"
DATA_DIR="$NODE_DIR/data"

echo ""
echo "=========================================="
echo "  WELCOME TO OUROBOROS NETWORK"
echo "=========================================="
echo ""
echo "Setting up your validator node..."

# Create directories
mkdir -p "$NODE_DIR" "$DATA_DIR"

# Download binary
echo ""
echo -n "[1/5] Downloading node binary..."
ARCH=$(uname -m)
if [ "$ARCH" = "x86_64" ]; then
    URL="https://github.com/ipswyworld/ouroboros/releases/latest/download/ouro-node-linux-x86_64"
elif [ "$ARCH" = "aarch64" ]; then
    URL="https://github.com/ipswyworld/ouroboros/releases/latest/download/ouro-node-linux-aarch64"
else
    echo " Unsupported architecture: $ARCH"
    exit 1
fi

if curl -sL "$URL" -o "$NODE_DIR/ouro-node" 2>/dev/null; then
    chmod +x "$NODE_DIR/ouro-node"
    echo " Done"
else
    echo " Failed"
    exit 1
fi

# Create wallet
echo -n "[2/5] Creating wallet..."
WALLET_ADDR="0x$(openssl rand -hex 20)"
NODE_ID="ouro_$(cat /dev/urandom | tr -dc 'a-z0-9' | fold -w 12 | head -n 1)"
echo "$WALLET_ADDR" > "$NODE_DIR/wallet.txt"
echo "$NODE_ID" > "$NODE_DIR/node_id.txt"
echo " Done"

# Create config
echo -n "[3/5] Configuring node..."
cat > "$NODE_DIR/.env" <<ENVEOF
ROCKSDB_PATH=$DATA_DIR
STORAGE_MODE=rocks
RUST_LOG=info
API_ADDR=0.0.0.0:8001
LISTEN_ADDR=0.0.0.0:9001
NODE_ID=$NODE_ID
SEED_NODES=136.112.101.176:9001
ENVEOF
echo " Done"

# Create systemd service
echo -n "[4/5] Configuring auto-start..."
sudo tee /etc/systemd/system/ouroboros.service > /dev/null <<SVCEOF
[Unit]
Description=Ouroboros Node
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$NODE_DIR
ExecStart=$NODE_DIR/ouro-node start
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
SVCEOF

sudo systemctl daemon-reload
sudo systemctl enable ouroboros.service
echo " Done"

# Create ouro CLI
sudo tee /usr/local/bin/ouro > /dev/null <<'CLIEOF'
#!/bin/bash
NODE_DIR="$HOME/.ouroboros"

case "$1" in
    status)
        echo ""
        echo "=========================================="
        echo "    OUROBOROS NODE STATUS"
        echo "=========================================="
        [ -f "$NODE_DIR/node_id.txt" ] && NODE_ID=$(cat "$NODE_DIR/node_id.txt") || NODE_ID="Unknown"
        [ -f "$NODE_DIR/wallet.txt" ] && WALLET=$(cat "$NODE_DIR/wallet.txt") || WALLET="Unknown"

        if systemctl is-active --quiet ouroboros; then
            echo "Status: RUNNING"
        else
            echo "Status: STOPPED"
        fi
        echo "Node ID: $NODE_ID"
        echo "Wallet: $WALLET"
        echo ""

        if curl -sf http://localhost:8001/health >/dev/null 2>&1; then
            echo "API: http://localhost:8001"
            echo ""
            echo "Check rewards: ouro rewards"
            echo "Wallet balance: ouro wallet balance"
        else
            echo "API: Offline"
        fi
        echo ""
        echo "Commands: ouro start|stop|wallet|rewards"
        echo "=========================================="
        ;;
    start)
        echo "Starting Ouroboros node..."
        sudo systemctl start ouroboros
        sleep 3
        ouro status
        ;;
    stop)
        echo "Stopping Ouroboros node..."
        sudo systemctl stop ouroboros
        echo "Node stopped"
        ;;
    wallet)
        if [ "$2" = "balance" ]; then
            WALLET=$(cat "$NODE_DIR/wallet.txt" 2>/dev/null || echo "Not created")
            echo "Checking balance for $WALLET..."
            curl -s "http://localhost:8001/balance/$WALLET"
        else
            WALLET=$(cat "$NODE_DIR/wallet.txt" 2>/dev/null || echo "Not created")
            echo "Your Wallet: $WALLET"
            echo ""
            echo "Commands:"
            echo "  ouro wallet balance  - Check balance"
        fi
        ;;
    rewards)
        NODE_ID=$(cat "$NODE_DIR/node_id.txt" 2>/dev/null || echo "Unknown")
        echo "Fetching rewards for $NODE_ID..."
        curl -s "http://localhost:8001/metrics/$NODE_ID"
        ;;
    *)
        echo ""
        echo "Ouroboros Node CLI"
        echo ""
        echo "Usage: ouro [command]"
        echo ""
        echo "Commands:"
        echo "  status   - Show node status"
        echo "  start    - Start node"
        echo "  stop     - Stop node"
        echo "  wallet   - Show wallet address"
        echo "  rewards  - Check earned rewards"
        echo ""
        ;;
esac
CLIEOF

sudo chmod +x /usr/local/bin/ouro

# Start node
echo -n "[5/5] Starting node..."
sudo systemctl start ouroboros
sleep 3
echo " Done"
echo ""

# Success message
echo "=========================================="
echo "  SUCCESS! You're now validating!"
echo "=========================================="
echo ""
echo "Your Node:"
echo "   Node ID: $NODE_ID"
echo "   Wallet:  $WALLET_ADDR"
echo "   Status:  http://localhost:8001/health"
echo ""
echo "Earnings:"
echo "   ~4.5 OURO/hour (based on uptime + validations)"
echo "   Check rewards: ouro rewards"
echo ""
echo "Wallet saved to: $NODE_DIR/wallet.txt"
echo "   Backup this file - you'll need it to recover funds!"
echo ""
echo "Quick Commands:"
echo "   ouro status   - Live node status"
echo "   ouro wallet   - Your wallet address"
echo "   ouro rewards  - Check earnings"
echo ""
echo "Your node will auto-start on boot"
echo "Keep your system online to maximize rewards!"
echo ""
