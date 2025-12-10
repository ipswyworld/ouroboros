# Ouroboros Network Information

Real-time network status and seed node information.

---

## Seed Nodes

### Primary Seed Node
```
Address: 34.171.88.26:9001
BFT Port: 34.171.88.26:9091
API: http://34.171.88.26:8001
Region: US Central (GCP)
Status: ✅ Online
```

### How to Connect
Your node automatically connects to the seed node when you run the join script. No manual configuration needed!

---

## Network Statistics

Check real-time network stats via the API:

### Total Validators
```bash
curl http://34.171.88.26:8001/metrics/leaderboard | jq 'length'
```

### Top Validators
```bash
curl http://34.171.88.26:8001/metrics/leaderboard | jq '.[0:10]'
```

### Network Health
```bash
curl http://34.171.88.26:8001/health
```

---

## Network Architecture

### Node Types

**1. Heavy Validators (PostgreSQL)**
- Full blockchain history
- Advanced querying capabilities
- Best for: Servers, data centers
- Storage backend: PostgreSQL

**2. Light Validators (RocksDB)**
- Lightweight embedded storage
- No external database required
- Best for: Community nodes, desktops
- Storage backend: RocksDB

Both node types participate equally in consensus!

---

## Network Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 8001 | HTTP | REST API |
| 9001 | TCP | P2P Networking |
| 9091 | TCP | BFT Consensus |

**Firewall Configuration:**
- Inbound: Allow 9001, 9091
- Outbound: Allow all

---

## Consensus Mechanism

**HotStuff BFT** with leader rotation:
- 3-phase commit (Prepare, Pre-commit, Commit)
- Byzantine fault tolerance: tolerates up to 1/3 malicious nodes
- Finality: Immediate (no probabilistic finality)
- Block time: ~2-5 seconds

---

## Reward Distribution

Rewards are calculated and distributed automatically:

| Action | Reward | Frequency |
|--------|--------|-----------|
| Propose Block | 20 OURO | Per block |
| Validate Block | 3 OURO | Per vote |
| Network Uptime | 1.5 OURO/hr | Hourly |

**View Your Earnings:**
```bash
curl http://localhost:8001/rewards/YOUR_ADDRESS
```

---

## Network Growth

Want to add more seed nodes or validator infrastructure?
1. Deploy a heavy validator with PostgreSQL
2. Open an issue to list your node as a seed
3. Community validators will connect to you

---

## Network Upgrades

Major network upgrades will be announced via:
- GitHub Releases
- Network-wide announcements
- Validator consensus votes

**Stay Updated:**
- Watch this repository for releases
- Check `/health` endpoint regularly
- Monitor node logs for warnings

---

## Troubleshooting

### Can't Connect to Seed Node
```bash
# Test connectivity
curl http://34.171.88.26:8001/health

# Check your firewall
sudo iptables -L | grep 9001
```

### Node Not Syncing
1. Check logs: `tail -f ~/.ouroboros/node.log`
2. Verify network connectivity
3. Restart node: `systemctl --user restart ouroboros-node`

### Rewards Not Updating
- Rewards are calculated hourly (uptime) and per-block (validation)
- Check your metrics: `curl http://localhost:8001/metrics/YOUR_ADDRESS`
- Ensure node is actively participating in consensus

---

**Need Help?** Open an issue on GitHub with your node logs and error messages.
