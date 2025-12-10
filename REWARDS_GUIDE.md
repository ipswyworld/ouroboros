# Ouroboros Rewards Guide

Learn how validators earn OURO coins and maximize your rewards.

---

## Reward Structure

Ouroboros rewards validators for three types of contributions:

### 1. Block Proposals (20 OURO)
**What**: Creating and proposing new blocks to the network
**When**: Each time your node is selected as leader and proposes a block
**Frequency**: Depends on leader rotation and network activity

**How to Maximize:**
- Keep your node online 24/7
- Ensure stable network connection
- Maintain fast block creation times

### 2. Block Validation (3 OURO)
**What**: Voting to accept or reject proposed blocks
**When**: Each time you vote on another validator's block proposal
**Frequency**: Continuous during normal operation

**How to Maximize:**
- Stay synchronized with the network
- Respond quickly to block proposals
- Maintain healthy peer connections

### 3. Uptime Bonus (1.5 OURO/hour)
**What**: Staying online and participating in the network
**When**: Calculated hourly for all active nodes
**Minimum**: Must be online for at least 1 hour to earn

**How to Maximize:**
- Set up your node as a systemd service (auto-restart)
- Monitor node health regularly
- Ensure your system doesn't sleep/hibernate

---

## Reward Calculation Examples

### Example 1: Active Validator (24 hours)
```
Blocks Proposed: 10 blocks × 20 OURO = 200 OURO
Block Validations: 500 votes × 3 OURO = 1,500 OURO
Uptime Bonus: 24 hours × 1.5 OURO = 36 OURO
-------------------------------------------------
Total Earned: 1,736 OURO per day
```

### Example 2: Moderate Validator (24 hours)
```
Blocks Proposed: 5 blocks × 20 OURO = 100 OURO
Block Validations: 300 votes × 3 OURO = 900 OURO
Uptime Bonus: 24 hours × 1.5 OURO = 36 OURO
-------------------------------------------------
Total Earned: 1,036 OURO per day
```

### Example 3: Light Validator (part-time, 8 hours)
```
Blocks Proposed: 1 block × 20 OURO = 20 OURO
Block Validations: 80 votes × 3 OURO = 240 OURO
Uptime Bonus: 8 hours × 1.5 OURO = 12 OURO
-------------------------------------------------
Total Earned: 272 OURO per day
```

---

## Factors Affecting Earnings

### Network Activity
- **High Activity**: More blocks = more proposals and validations
- **Low Activity**: Fewer opportunities to earn

### Your Node's Performance
- **Fast Response**: More likely to be selected as leader
- **Reliable Uptime**: Consistent earnings from uptime bonus
- **Good Connectivity**: More block validations

### Number of Validators
- **Fewer Validators**: More proposals per node (higher earnings)
- **More Validators**: More distributed rewards (lower per-node earnings)

---

## Checking Your Rewards

### Real-Time Metrics
```bash
# Get your current metrics
curl http://localhost:8001/metrics/YOUR_ADDRESS | jq

# Calculate earnings rate (OURO per hour)
curl http://localhost:8001/metrics/YOUR_ADDRESS | \
  jq '.total_rewards / (.uptime_seconds / 3600)'
```

### Reward History
```bash
# View recent rewards
curl http://localhost:8001/rewards/YOUR_ADDRESS | jq '.[0:20]'

# Count rewards by type
curl http://localhost:8001/rewards/YOUR_ADDRESS | \
  jq 'group_by(.reward_type) | map({type: .[0].reward_type, count: length, total: map(.amount) | add})'
```

### Leaderboard Position
```bash
# See where you rank
curl http://localhost:8001/metrics/leaderboard | \
  jq 'to_entries | map({rank: .key + 1, address: .value.node_address, rewards: .value.total_rewards})'
```

---

## Optimization Tips

### 1. Maximize Uptime
**Setup:**
```bash
# Linux: Use systemd (automatic restart)
systemctl --user enable ouroboros-node
systemctl --user start ouroboros-node

# Windows: Use Task Scheduler for auto-start
```

**Monitoring:**
```bash
# Check if your node is running
curl http://localhost:8001/health

# Monitor uptime
curl http://localhost:8001/metrics/YOUR_ADDRESS | jq '.uptime_seconds / 3600'
```

### 2. Optimize Network Performance
- **Low Latency**: Host in datacenter or on fast home connection
- **Open Ports**: Ensure ports 9001, 9091 are accessible
- **Stable IP**: Use static IP or DDNS for better peer discovery

### 3. Monitor and Maintain
```bash
# Check for errors daily
tail -100 ~/.ouroboros/node.log | grep ERROR

# Restart if stuck
systemctl --user restart ouroboros-node

# Check disk space
df -h ~/.ouroboros/data
```

### 4. Join Early
- Early validators earn more due to fewer competitors
- Network growth dilutes per-node rewards over time
- First-mover advantage in decentralized networks

---

## Reward Distribution Timeline

| Action | Timing | Verification |
|--------|--------|--------------|
| Block Proposal | Immediate | Check `/rewards/:address` |
| Block Validation | Immediate | Check `/rewards/:address` |
| Uptime Bonus | Hourly | Runs on the hour (XX:00) |
| Leaderboard Update | Real-time | Check `/metrics/leaderboard` |

---

## Fair Reward Principles

### Transparency
- All rewards are publicly visible
- Anyone can query any validator's metrics
- Reward calculations are deterministic

### Meritocracy
- Rewards based solely on contributions
- No special treatment for any validator
- Leader rotation ensures fairness

### Adjustability
Reward parameters are stored in the database and can be adjusted via governance:

```sql
-- View current reward config
SELECT * FROM reward_config;

-- Example output:
-- block_proposal_reward:    20 OURO
-- block_validation_reward:  3 OURO
-- uptime_reward_per_hour:   1.5 OURO
```

---

## Common Questions

### Q: When do I receive my OURO coins?
**A**: Rewards are recorded immediately in your validator metrics. The `total_rewards` field shows your cumulative earnings. OURO coin transfers to your wallet will be implemented in a future update.

### Q: Can I withdraw my OURO coins?
**A**: Currently, rewards are tracked on-chain but not yet transferable. Wallet integration and coin transfers are planned for future releases.

### Q: Why did my earnings drop?
**A**: Common reasons:
- More validators joined (rewards distributed to more nodes)
- Your node went offline temporarily
- Network activity decreased
- Leader rotation selected other validators

### Q: How do I increase my block proposal count?
**A**: Block proposals depend on:
- Your node being online and healthy
- Leader rotation algorithm (fair rotation among all validators)
- Network activity level
- You cannot game the system - proposals are randomly rotated

### Q: Are rewards guaranteed?
**A**: No. Rewards depend on network participation, activity levels, and your node's uptime. This is a decentralized network with no central authority guaranteeing payments.

---

## Future Reward Enhancements

Planned features (not yet implemented):

- **Wallet Integration**: Transfer OURO coins to external wallets
- **Staking Rewards**: Lock OURO coins to earn additional yield
- **Delegation**: Allow others to delegate to your validator
- **Slashing**: Penalties for malicious behavior (Byzantine faults)
- **Governance Voting**: Use OURO coins to vote on protocol changes

---

## Track Your Progress

### Daily Checklist
- [ ] Check node is running: `curl http://localhost:8001/health`
- [ ] View today's earnings: `curl http://localhost:8001/rewards/YOUR_ADDRESS`
- [ ] Check leaderboard rank: `curl http://localhost:8001/metrics/leaderboard`
- [ ] Monitor logs for errors: `tail ~/.ouroboros/node.log`

### Weekly Review
- [ ] Total rewards earned this week
- [ ] Average OURO per hour
- [ ] Blocks proposed vs other validators
- [ ] Node uptime percentage
- [ ] Network growth (total validators)

---

## Support

**Questions about rewards?**
- Check your metrics: `/metrics/:address`
- View reward history: `/rewards/:address`
- Open a GitHub issue if you suspect incorrect calculations

**Remember**: Ouroboros is a decentralized network. Your earnings depend on your participation and the overall network health. Stay active, stay online, earn OURO! 🚀
