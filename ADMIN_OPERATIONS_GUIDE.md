# Admin Operations Guide - Fraud Detection System

**Quick reference for administrators responding to fraud alerts.**

---

## Table of Contents

1. [Alert Response Procedures](#alert-response-procedures)
2. [Common Scenarios](#common-scenarios)
3. [API Commands](#api-commands)
4. [Investigation Tools](#investigation-tools)
5. [Escalation Procedures](#escalation-procedures)

---

## Alert Response Procedures

### When You Receive an Alert

**1. Check Alert Severity**

```bash
# Get recent alerts
curl https://your-node.run.app/fraud/alerts

# Get critical alerts only
curl https://your-node.run.app/fraud/alerts/critical
```

**2. Review Alert Details**

Each alert contains:
- `alert_id` - Unique identifier
- `severity` - Low, Medium, High, Critical
- `alert_type` - Type of fraud detected
- `entity` - Affected relayer/operator/user
- `description` - What triggered the alert
- `timestamp` - When it occurred
- `auto_action` - What the system did automatically

**3. Determine Response Based on Severity**

| Severity | Response Time | Action Required |
|----------|---------------|-----------------|
| **Critical** | Immediate (< 5 min) | Investigate + Take action |
| **High** | Within 30 min | Investigate + Monitor |
| **Medium** | Within 2 hours | Review + Document |
| **Low** | Within 24 hours | Review logs |

---

## Common Scenarios

### Scenario 1: High Failure Rate Alert

**Alert**: `SuspiciousRelayPattern` - Relayer has >10% failure rate

**Symptoms**:
```json
{
  "severity": "High",
  "alert_type": "SuspiciousRelayPattern",
  "entity": "relayer_xyz",
  "description": "High failure rate: 25.00%"
}
```

**Response**:

1. **Check relayer statistics**:
   ```bash
   curl https://your-node.run.app/fraud/stats/relayer_xyz
   ```

2. **Investigate failures**:
   - Review logs for error patterns
   - Check if source chain is accessible
   - Verify merkle proofs are valid

3. **Actions**:
   - **If malicious**: Blacklist relayer
   - **If technical issue**: Contact relayer to fix
   - **If temporary glitch**: Monitor for improvement

4. **Blacklist if needed**:
   ```bash
   curl -X POST https://your-node.run.app/fraud/blacklist/relayer_xyz \
     -H "Content-Type: application/json" \
     -d '{"reason": "Sustained high failure rate", "permanent": false}'
   ```

---

### Scenario 2: Double Spend Detected

**Alert**: `DoubleSpendAttempt` - User trying to spend same nonce twice

**Symptoms**:
```json
{
  "severity": "Critical",
  "alert_type": "DoubleSpendAttempt",
  "entity": "user_abc",
  "description": "Duplicate nonce detected: 42"
}
```

**Response**:

1. **Immediate action - System auto-submits fraud proof**
   - The fraud detection system automatically submits fraud proof
   - Verify submission succeeded

2. **Freeze user account**:
   ```bash
   curl -X POST https://your-node.run.app/fraud/blacklist/user_abc \
     -H "Content-Type: application/json" \
     -d '{"reason": "Double spend attempt detected", "permanent": true}'
   ```

3. **Investigate extent**:
   - Check all transactions from this user
   - Look for coordinated attacks (multiple users)
   - Review transaction history

4. **Document incident**:
   - Record all evidence
   - Save transaction hashes
   - Note timestamps and amounts

5. **Report if needed**:
   - If large amount: Report to authorities
   - If coordinated: Alert other validators

---

### Scenario 3: Missing State Anchor

**Alert**: `MissingStateAnchor` - Microchain operator hasn't anchored state

**Symptoms**:
```json
{
  "severity": "High",
  "alert_type": "MissingStateAnchor",
  "entity": "operator_microchain_123",
  "description": "Microchain microchain_123 has not anchored state in 6 hours"
}
```

**Response**:

1. **Contact operator**:
   - Email/Slack notification
   - Check if planned maintenance
   - Verify operator is reachable

2. **Monitor microchain**:
   - Check if transactions are still processing
   - Verify no user complaints
   - Look for signs of operator exit scam

3. **If operator unresponsive** (>24 hours):
   - Alert all microchain users
   - Guide users to force exit
   - Consider operator slashing

4. **Help users force exit**:
   ```bash
   # Users should have their merkle proofs
   # Guide them through force exit process
   # See FRAUD_PROOF_SYSTEM.md for details
   ```

---

### Scenario 4: Abnormal Volume

**Alert**: `AbnormalVolume` - Entity exceeded volume threshold

**Symptoms**:
```json
{
  "severity": "High",
  "alert_type": "AbnormalVolume",
  "entity": "relayer_xyz",
  "description": "Abnormal volume: 5000 OURO"
}
```

**Response**:

1. **Check if legitimate**:
   - Large merchant settlement?
   - Known high-volume user?
   - Authorized bulk transfer?

2. **Verify all transfers**:
   - Sample check recent transactions
   - Verify recipients exist
   - Check for wash trading patterns

3. **Actions**:
   - **If legitimate**: Whitelist or increase threshold
   - **If suspicious**: Increase monitoring level
   - **If fraudulent**: Freeze and investigate

4. **Increase monitoring**:
   ```bash
   # Monitoring level automatically increased
   # Review logs more frequently
   # Set up real-time alerts for this entity
   ```

---

### Scenario 5: Blacklisted Entity Attempting Action

**Alert**: Entity on blacklist trying to relay/operate

**Symptoms**:
```json
{
  "severity": "Critical",
  "alert_type": "SuspiciousRelayPattern",
  "entity": "malicious_relayer",
  "description": "Blacklisted entity attempting relay",
  "auto_action": "PauseRelayer"
}
```

**Response**:

1. **Verify blacklist status**:
   ```bash
   curl https://your-node.run.app/fraud/blacklist/malicious_relayer
   ```

2. **System auto-blocks** - No action needed
   - Request is automatically rejected
   - Entity cannot perform action

3. **Investigate**:
   - Why is blacklisted entity still trying?
   - Are they using multiple accounts?
   - Need to block IP/region?

4. **Escalate if persistent**:
   - Report to network coordinators
   - Consider network-level ban
   - Document pattern for future

---

## API Commands

### Check System Status

```bash
# Fraud detection status
curl https://your-node.run.app/fraud/status

# Full monitoring report
curl https://your-node.run.app/fraud/report
```

### Query Alerts

```bash
# Get 50 most recent alerts
curl https://your-node.run.app/fraud/alerts

# Get critical alerts only
curl https://your-node.run.app/fraud/alerts/critical

# Get specific alert details
# (Check logs for specific alert ID)
```

### Manage Blacklist

```bash
# Check if entity is blacklisted
curl https://your-node.run.app/fraud/blacklist/ENTITY_ID

# Add to blacklist
curl -X POST https://your-node.run.app/fraud/blacklist/ENTITY_ID \
  -H "Content-Type: application/json" \
  -d '{
    "reason": "Your reason here",
    "permanent": false
  }'

# Remove from blacklist (requires manual database update)
# Contact system administrator
```

### Get Entity Statistics

```bash
# Get statistics for relayer/operator/user
curl https://your-node.run.app/fraud/stats/ENTITY_ID

# Response shows:
# - total_relays
# - successful_relays
# - failed_relays
# - total_volume
# - success_rate
```

---

## Investigation Tools

### 1. Log Analysis

```bash
# View fraud detection logs (Cloud Run)
gcloud run services logs read ouroboros-node \
  --region=us-central1 \
  | grep -i fraud

# Filter for specific entity
gcloud run services logs read ouroboros-node \
  --region=us-central1 \
  | grep "relayer_xyz"

# Last 1 hour only
gcloud run services logs read ouroboros-node \
  --region=us-central1 \
  --start-time=$(date -u -d '1 hour ago' '+%Y-%m-%dT%H:%M:%S')
```

### 2. Metrics Dashboard

```bash
# View metrics (if configured)
gcloud monitoring dashboards list

# Open dashboard in browser
# Check for spike in alert counts
```

### 3. Database Queries

```sql
-- Check recent relays
SELECT * FROM relays
WHERE relayer = 'relayer_xyz'
ORDER BY timestamp DESC
LIMIT 100;

-- Check failure rate
SELECT
  relayer,
  COUNT(*) as total,
  SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful,
  AVG(CASE WHEN success THEN 1.0 ELSE 0.0 END) * 100 as success_rate
FROM relays
GROUP BY relayer
HAVING success_rate < 90
ORDER BY success_rate ASC;
```

### 4. Network Analysis

```bash
# Check peer connections
curl https://your-node.run.app/network/peers

# Verify consensus status
curl https://your-node.run.app/bft/status

# Check mempool
curl https://your-node.run.app/api/mempool
```

---

## Escalation Procedures

### Level 1: Automated Response (System Handles)

**Examples**:
- Low/Medium severity alerts
- Temporary issues
- Single occurrence

**Action**: System auto-responds, admin reviews logs

---

### Level 2: Admin Investigation (< 30 min)

**Examples**:
- High severity alerts
- Repeated low/medium alerts from same entity
- Volume anomalies

**Action**:
1. Review alert details
2. Check entity statistics
3. Investigate logs
4. Take corrective action (blacklist, contact entity, etc.)
5. Document findings

---

### Level 3: Immediate Response (< 5 min)

**Examples**:
- Critical severity alerts
- Double spend attempts
- Blacklisted entity breaching security
- System-wide fraud pattern

**Action**:
1. **Immediately**: Review alert
2. **Within 2 min**: Verify auto-actions succeeded
3. **Within 5 min**: Manual intervention if needed
4. **Within 15 min**: Document incident
5. **Within 1 hour**: Notify team/stakeholders

---

### Level 4: Emergency Response (Immediate)

**Examples**:
- Coordinated attack across multiple entities
- Large-scale fund theft
- System compromise
- Consensus failure

**Action**:
1. **Immediately**: Alert all admins
2. **Emergency meeting**: Assemble response team
3. **Consider**: Pause affected services
4. **Coordinate**: With other validators/nodes
5. **Communicate**: Update users on status
6. **Document**: Everything for post-mortem

**Emergency Contacts**:
- Tech Lead: [contact info]
- Security Lead: [contact info]
- On-Call Engineer: [contact info]

---

## Daily Operations Checklist

### Morning Check (Start of Day)

- [ ] Review overnight alerts
- [ ] Check fraud detection status
- [ ] Review monitoring dashboard
- [ ] Check critical alerts (should be 0)
- [ ] Review blacklist (any additions?)
- [ ] Check system health metrics

### Throughout Day

- [ ] Monitor real-time alerts
- [ ] Respond to notifications
- [ ] Investigate high-priority items
- [ ] Document actions taken

### End of Day

- [ ] Review all alerts from day
- [ ] Document unresolved issues
- [ ] Update incident log
- [ ] Brief next shift (if 24/7)
- [ ] Clean up old alerts (system auto-does this)

---

## Weekly Operations Checklist

- [ ] Review fraud detection configuration
- [ ] Analyze fraud trends
- [ ] Check if thresholds need adjustment
- [ ] Review blacklist (remove temporary bans if appropriate)
- [ ] Generate weekly report
- [ ] Team review meeting
- [ ] Update documentation if needed

---

## Monthly Operations Checklist

- [ ] Full fraud system audit
- [ ] Review all permanent blacklists
- [ ] Analyze false positive rate
- [ ] Tune detection thresholds
- [ ] Update runbooks based on learnings
- [ ] Security review
- [ ] Backup verification

---

## Best Practices

### DO ✅

- **Respond to critical alerts immediately**
- **Document all investigations**
- **Keep detailed incident logs**
- **Review trends weekly**
- **Test fraud detection regularly**
- **Keep contact info updated**
- **Communicate with affected parties**

### DON'T ❌

- **Ignore low severity alerts** - They can indicate larger issues
- **Blacklist without investigation** - Could be false positive
- **Make permanent decisions alone** - Consult team for permanent bans
- **Disable fraud detection** - Even temporarily in production
- **Share sensitive alert details** - Keep confidential
- **Panic** - Follow procedures calmly

---

## Contact Information

**Internal Team**:
- Tech Lead: [email/phone]
- Security Team: [email/phone]
- On-Call: [rotation schedule]

**External**:
- Validator Network: [contact]
- Security Researchers: [bug bounty]

**Escalation Path**:
1. On-Call Engineer
2. Tech Lead
3. Security Lead
4. CTO/CEO

---

## Appendix: Alert Type Reference

| Alert Type | Severity | Common Cause | Response Time |
|------------|----------|--------------|---------------|
| SuspiciousRelayPattern | High | High failure rate | 30 min |
| DoubleSpendAttempt | Critical | Malicious user | Immediate |
| HighValueUnvalidated | Critical | Large failed relay | Immediate |
| MissingStateAnchor | High | Operator offline | 30 min |
| InvalidStateTransition | Critical | Operator fraud | Immediate |
| AbnormalVolume | High | Unusual activity | 30 min |
| RapidWithdrawal | Medium | Potential exit scam | 2 hours |
| RepeatedFailedProofs | Medium | Technical issues | 2 hours |

---

**Document Version**: 1.0
**Last Updated**: 2025-12-23
**Next Review**: Monthly
