# Ouroboros Testing Guide
## Phase 4A & 4B Feature Verification

This guide will help you test all the features implemented in Phase 4A (Critical Security) and Phase 4B (Critical Features).

---

## Prerequisites

✅ PostgreSQL database running (via docker-compose)
✅ Database migrations completed
✅ Project built successfully (`cargo build`)

---

## Quick Start

### 1. Start the Application

**Option A: Use test environment configuration**
```bash
cd ouro_dag
cp .env.test .env
cargo run
```

**Option B: Use existing .env with API keys**
```bash
cd ouro_dag
# Edit .env to add: API_KEYS=test-key-123,admin-key-456
cargo run
```

The server should start on `http://localhost:8001` (or the `API_ADDR` you configured).

### 2. Run the Test Script

In a new terminal:
```bash
cd ouro_dag
chmod +x test_features.sh
./test_features.sh
```

---

## Manual Testing

If you prefer to test features manually, here's a comprehensive guide:

### A. API Authentication Testing

#### Test 1: Valid API Key (Should Succeed)
```bash
curl -X POST http://localhost:8001/tx/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key-123" \
  -d '{"sender":"alice","recipient":"bob","amount":1000,"fee":10}'
```

**Expected Result:** HTTP 200/201 or valid application response

#### Test 2: Invalid API Key (Should Fail)
```bash
curl -X POST http://localhost:8001/tx/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: wrong-key" \
  -d '{"sender":"alice","recipient":"bob","amount":1000,"fee":10}'
```

**Expected Result:** HTTP 401/403 Unauthorized

#### Test 3: Missing API Key (Should Fail)
```bash
curl -X POST http://localhost:8001/tx/submit \
  -H "Content-Type: application/json" \
  -d '{"sender":"alice","recipient":"bob","amount":1000,"fee":10}'
```

**Expected Result:** HTTP 401/403 Unauthorized

---

### B. Rate Limiting Testing

Send multiple requests rapidly to trigger rate limiting:

```bash
for i in {1..20}; do
  echo "Request $i:"
  curl -w "\nHTTP Code: %{http_code}\n" \
    -X POST http://localhost:8001/tx/submit \
    -H "Content-Type: application/json" \
    -H "X-API-Key: test-key-123" \
    -d "{\"sender\":\"alice\",\"recipient\":\"bob\",\"amount\":$i,\"fee\":1}"
  echo "---"
  sleep 0.1
done
```

**Expected Result:** After several requests, you should see HTTP 429 (Too Many Requests)

---

### C. Merkle Proof Generation Testing

1. Submit a transaction:
```bash
curl -X POST http://localhost:8001/tx/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key-123" \
  -d '{"sender":"alice","recipient":"bob","amount":5000,"fee":50}' | jq
```

2. Extract the `tx_hash` from the response

3. Query the transaction (after it's included in a block):
```bash
curl http://localhost:8001/tx/{TX_HASH_HERE} \
  -H "X-API-Key: test-key-123" | jq
```

**Expected Result:** Response should include `merkle_proof` field with:
- `root`: Merkle root hash
- `index`: Transaction position in block
- `path`: Array of sibling hashes for verification

---

### D. Mempool Fee Prioritization Testing

Submit multiple transactions with different fees:

```bash
# Low fee transaction
curl -X POST http://localhost:8001/tx/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key-123" \
  -d '{"sender":"alice","recipient":"bob","amount":100,"fee":1}'

# Medium fee transaction
curl -X POST http://localhost:8001/tx/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key-123" \
  -d '{"sender":"alice","recipient":"charlie","amount":100,"fee":50}'

# High fee transaction
curl -X POST http://localhost:8001/tx/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key-123" \
  -d '{"sender":"alice","recipient":"dave","amount":100,"fee":100}'
```

**Expected Behavior:** When blocks are created, high-fee transactions should be prioritized and included first.

---

### E. Balance Tracking Testing

1. Submit several transactions:
```bash
curl -X POST http://localhost:8001/tx/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key-123" \
  -d '{"sender":"alice","recipient":"bob","amount":1000,"fee":10}'
```

2. Check balances in the database:
```bash
psql postgres://ouro:ouro_pass@localhost:15432/ouro_db \
  -c "SELECT account, balance, updated_at FROM balances ORDER BY updated_at DESC;"
```

**Expected Result:**
- Alice's balance should decrease by (amount + fee)
- Bob's balance should increase by amount
- All updates tracked with timestamps

---

### F. Peer Authentication Testing

This requires running multiple nodes with P2P connections.

1. **Authorized Peer Test:**
   - Set `AUTHORIZED_PEERS=node1,node2` in .env
   - Start node with `cargo run`
   - Connect from peer with `node_id=node1`
   - **Expected:** Connection accepted

2. **Unauthorized Peer Test:**
   - Same configuration
   - Connect from peer with `node_id=unauthorized-node`
   - **Expected:** Connection rejected with error message

You can check the logs for authorization messages:
```
✅ Peer node1 authenticated and authorized
🚨 Peer unauthorized-node not in authorized list - connection rejected
```

---

### G. TLS Enforcement in Production

1. Set production mode WITHOUT TLS:
```bash
ENVIRONMENT=production cargo run
```

**Expected Result:** Application should exit immediately with error:
```
🚨 CRITICAL: Production deployment REQUIRES TLS/HTTPS!
```

2. Set production mode WITH TLS:
```bash
ENVIRONMENT=production \
TLS_CERT_PATH=/path/to/cert.pem \
TLS_KEY_PATH=/path/to/key.pem \
cargo run
```

**Expected Result:** Application starts successfully with TLS enabled

---

### H. Alerting System Testing

1. Start a simple webhook receiver (for testing):
```bash
# In a new terminal
python3 -m http.server 9999
```

2. Configure alerting in .env:
```bash
ALERT_ENABLED=true
ALERT_WEBHOOK_URL=http://localhost:9999/webhook
```

3. Trigger an alert (e.g., by causing a batch to fail MAX_RETRIES times)

**Expected Result:**
- Alert logged to console
- Webhook POST request sent to configured URL
- Check webhook receiver logs for incoming alert

---

### I. Binary Consensus Message Format

The consensus messages now use binary `Vec<u8>` format instead of hex strings for 32-40% bandwidth savings.

**To verify:**
1. Enable BFT consensus (if not running)
2. Check network traffic between nodes
3. Inspect serialized message sizes

**Expected:** Signature fields are binary encoded, not hex strings

---

## Database Inspection

### View Schema
```bash
psql postgres://ouro:ouro_pass@localhost:15432/ouro_db \
  -c "\dt"
```

### Check Balances
```bash
psql postgres://ouro:ouro_pass@localhost:15432/ouro_db \
  -c "SELECT * FROM balances ORDER BY updated_at DESC LIMIT 10;"
```

### Check Transactions
```bash
psql postgres://ouro:ouro_pass@localhost:15432/ouro_db \
  -c "SELECT * FROM tx_index ORDER BY created_at DESC LIMIT 10;"
```

### Check Migrations
```bash
psql postgres://ouro:ouro_pass@localhost:15432/ouro_db \
  -c "SELECT * FROM schema_migrations ORDER BY applied_at DESC;"
```

---

## Troubleshooting

### Server won't start

1. **Check database connection:**
   ```bash
   psql postgres://ouro:ouro_pass@localhost:15432/ouro_db -c "SELECT 1;"
   ```

2. **Check port availability:**
   ```bash
   netstat -tuln | grep 8001
   ```

3. **Check environment variables:**
   ```bash
   cat .env
   ```

### API requests fail

1. **Verify API_KEYS are set:**
   ```bash
   grep API_KEYS .env
   ```

2. **Check server logs** for authentication errors

3. **Verify X-API-Key header** is included in requests

### Rate limiting not working

- Check if multiple requests are being sent quickly enough
- Default limits may be high - check configuration
- Verify rate limiting middleware is applied to endpoints

---

## Feature Checklist

### Phase 4A: Critical Security
- [ ] API authentication enforced on all protected endpoints
- [ ] Invalid/missing API keys rejected with 401/403
- [ ] TLS enforcement in production mode
- [ ] Graceful error when TLS not configured in production
- [ ] All API routers integrated with auth middleware

### Phase 4B: Critical Features
- [ ] Binary consensus message format (Vec<u8> signatures)
- [ ] Peer authentication with AUTHORIZED_PEERS
- [ ] P2P TLS infrastructure setup
- [ ] Merkle proof generation for transactions
- [ ] Mempool fee-based prioritization
- [ ] Balance tracking in PostgreSQL
- [ ] Monitoring and alerting system
- [ ] Rate limiting on all protected endpoints

---

## Next Steps

After verifying all features work correctly:

1. **Performance Testing**
   - Load testing with multiple concurrent requests
   - Measure transaction throughput
   - Monitor resource usage

2. **Integration Testing**
   - Multi-node P2P network testing
   - BFT consensus validation
   - End-to-end transaction flow

3. **Security Audit**
   - Penetration testing
   - Authentication bypass attempts
   - Rate limiting stress tests

4. **Documentation**
   - API documentation
   - Deployment guide
   - Operational runbook

5. **Deployment Preparation**
   - Docker images
   - Kubernetes manifests
   - Monitoring dashboards
   - Log aggregation setup
