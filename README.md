# Ouroboros Network

**A decentralized blockchain network with lightweight nodes and transparent rewards.**

🌐 Join the network in under 5 minutes | 💾 No database setup required | 🏆 Earn OURO coins for validation

---

## Quick Start

### Linux / macOS
```bash
bash <(curl -fsSL https://raw.githubusercontent.com/ipswyworld/ouroboros/main/join_ouroboros.sh)
```

### Windows (PowerShell as Administrator)
**Important**: Open **PowerShell** (not Command Prompt) as Administrator, then run:
```powershell
Invoke-WebRequest https://raw.githubusercontent.com/ipswyworld/ouroboros/main/join_ouroboros.ps1 | Invoke-Expression
```

That's it! Your node will automatically:
- Download the lightweight node binary (~19MB)
- Connect to the Ouroboros network
- Start validating transactions
- Earn OURO coin rewards

---

## What is Ouroboros?

Ouroboros is a **hybrid Byzantine Fault Tolerant (BFT) blockchain** that combines:
- **Lightweight nodes**: Run on any device with RocksDB embedded storage
- **Heavy validators**: Full nodes with PostgreSQL for blockchain history
- **Transparent rewards**: All validator contributions are publicly tracked
- **Decentralized P2P**: No central authority, fully distributed network

---

## Features

### 🚀 Easy to Join
No complicated setup. Download a binary, run one command, you're in.

### 💰 Earn Rewards
Validators earn OURO coins for:
- **Block proposals**: 20 OURO per block
- **Block validation**: 3 OURO per validation
- **Network uptime**: 1.5 OURO per hour

### 📊 Full Transparency
Check any validator's metrics:
```bash
# See validator contributions
curl http://localhost:8001/metrics/VALIDATOR_ADDRESS

# View leaderboard
curl http://localhost:8001/metrics/leaderboard

# Check your rewards
curl http://localhost:8001/rewards/YOUR_ADDRESS
```

### 🔒 Secure Consensus
HotStuff BFT consensus with:
- Post-quantum cryptography (Dilithium + Kyber)
- Leader rotation for fairness
- Byzantine fault tolerance

---

## Network Information

- **Seed Node**: `136.112.101.176:9001`
- **Network Status**: [NETWORK_INFO.md](NETWORK_INFO.md)
- **API Documentation**: [API_DOCUMENTATION.md](API_DOCUMENTATION.md)
- **Rewards Guide**: [REWARDS_GUIDE.md](REWARDS_GUIDE.md)

---

## System Requirements

### Lightweight Node (Community)
- **CPU**: 1 core
- **RAM**: 512MB
- **Storage**: 1GB
- **OS**: Linux, macOS, or Windows

### Heavy Validator (Server)
- **CPU**: 2+ cores
- **RAM**: 2GB+
- **Storage**: 10GB+
- **Database**: PostgreSQL 13+
- **OS**: Linux (Ubuntu 20.04+ recommended)

---

## Monitoring Your Node

### Check Node Status
```bash
curl http://localhost:8001/health
```

### View Logs
**Linux:**
```bash
tail -f ~/.ouroboros/node.log
```

**Windows:**
```powershell
Get-Content $env:USERPROFILE\.ouroboros\node.log -Tail 50 -Wait
```

### Check Your Rewards
```bash
# Replace YOUR_ADDRESS with your validator public key
curl http://localhost:8001/metrics/YOUR_ADDRESS
```

---

## Support & Community

- **Issues**: [GitHub Issues](https://github.com/ipswyworld/ouroboros/issues)
- **Network Stats**: Check `/metrics/leaderboard` endpoint
- **Updates**: Watch this repo for new releases

---

## Security

### Reporting Vulnerabilities
Please report security issues to the repository maintainer privately.

### Network Safety
- Never share your validator private keys
- Keep your node software updated
- Use firewall rules to protect your node

---

## License

This is a decentralized network. Binaries are provided as-is for network participation.

---

**Ready to join?** Run the quick start command for your platform above! 🚀
