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
ExecStart=/bin/sh -c 'sleep 30; /usr/local/bin/ouro-node start'
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload
sudo systemctl start ouro-node.service
journalctl -u ouro-node -n 50 --no-pager