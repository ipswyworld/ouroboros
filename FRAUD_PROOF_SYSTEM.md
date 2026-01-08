# Fraud Proof System

Comprehensive fraud detection and prevention system for Ouroboros blockchain.

## Table of Contents

1. [Overview](#overview)
2. [Cross-Chain Fraud Proofs](#cross-chain-fraud-proofs)
3. [Microchain Challenge System](#microchain-challenge-system)
4. [Fraud Detection Service](#fraud-detection-service)
5. [Usage Examples](#usage-examples)
6. [Security Parameters](#security-parameters)
7. [Testing](#testing)
8. [Production Deployment](#production-deployment)

---

## Overview

The Ouroboros fraud proof system provides three layers of security:

1. **Cross-Chain Fraud Proofs**: Detect and prove fraudulent cross-chain message relays
2. **Microchain Challenges**: Challenge invalid microchain state anchors
3. **Fraud Detection Service**: Automated monitoring and alerting system

### Key Features

✅ **Optimistic Relay Security**
- 10-minute challenge period for cross-chain transfers
- Economic incentives for fraud detection
- Automatic slashing of malicious relayers

✅ **Microchain State Validation**
- 7-day challenge period for state anchors
- Merkle proof verification
- Force exit mechanism for users

✅ **Continuous Monitoring**
- Real-time fraud pattern detection
- Automatic alerting and blacklisting
- Activity statistics and reporting

---

## Cross-Chain Fraud Proofs

The cross-chain fraud proof system protects optimistic relay transfers between subchains.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Cross-Chain Transfer                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Source Subchain ──────[Lock]──────> Message                │
│                                         │                     │
│                                         ▼                     │
│                         Relayer ───[Relay Message]───> Pool  │
│                                         │                     │
│                                         ▼                     │
│                              [Challenge Period: 10 min]      │
│                                         │                     │
│                          ┌──────────────┴──────────────┐     │
│                          │                             │     │
│                     [No Challenge]              [Fraud Proof]│
│                          │                             │     │
│                          ▼                             ▼     │
│                 [Confirm & Reward]           [Slash Relayer] │
│                          │                             │     │
│                          ▼                             ▼     │
│  Destination Subchain <─[Release]             [Revert TX]   │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### How It Works

#### 1. Relayer Submits Message

```rust
use ouro_dag::cross_chain::{FraudProofManager, CrossChainMessage};

let manager = FraudProofManager::new();

// Relayer deposits bond (100 OURO)
manager.deposit_bond("relayer1".to_string(), 100_000_000);

// Submit cross-chain message
let message = CrossChainMessage {
    source_subchain: "us_east".to_string(),
    destination_subchain: "africa_west".to_string(),
    sender: "alice".to_string(),
    recipient: "bob".to_string(),
    amount: 5_000_000, // 0.05 OURO
    nonce: 42,
    timestamp: current_time,
};

let message_hash = manager.submit_relay(
    message,
    "relayer1".to_string(),
    Some(merkle_proof),
    current_time,
).unwrap();

println!("✅ Message relayed: {:?}", hex::encode(message_hash));
println!("   Challenge period: 10 minutes");
```

#### 2. Anyone Can Submit Fraud Proof

```rust
use ouro_dag::cross_chain::FraudProofType;

// User detects fraud (insufficient balance on source)
let result = manager.submit_fraud_proof(
    message_hash,
    "challenger1".to_string(),
    FraudProofType::InsufficientBalance,
    vec![], // Evidence
    current_time + 300, // 5 minutes later
);

println!("⚠️  Fraud proof submitted!");
```

#### 3. System Verifies Proof

```rust
// Fetch source chain state
let mut source_state = HashMap::new();
source_state.insert("alice".to_string(), 1_000_000); // Alice only has 0.01 OURO

// Verify fraud proof
let is_valid = manager.verify_and_slash(
    message_hash,
    &source_state,
    &source_messages,
).unwrap();

if is_valid {
    println!("⚡ FRAUD PROVEN!");
    println!("   Relayer slashed: 100 OURO");
    println!("   Challenger rewarded: 50 OURO");
}
```

### Fraud Proof Types

| Type | Description | Evidence Required |
|------|-------------|-------------------|
| `MessageNotFound` | Message doesn't exist on source chain | Source chain state |
| `InsufficientBalance` | Sender lacks sufficient balance | Account balance proof |
| `InvalidMerkleProof` | Merkle proof doesn't verify | Correct merkle root |
| `DoubleRelay` | Same message relayed twice | Previous relay hash |

### Economic Security

| Parameter | Value | Purpose |
|-----------|-------|---------|
| Relayer Bond | 100 OURO | Stake required to relay |
| Challenge Period | 10 minutes | Time to submit fraud proof |
| Fraud Reward | 50 OURO | 50% of slashed bond |
| Confirmation Reward | 0.01 OURO | Bonus for honest relaying |

---

## Microchain Challenge System

The microchain challenge system allows users to challenge invalid state anchors and force exit from compromised microchains.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Microchain State Anchor                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Microchain ──[Anchor State Root]──> Mainchain              │
│                          │                                    │
│                          ▼                                    │
│               [Challenge Period: 7 days]                     │
│                          │                                    │
│                ┌─────────┴─────────┐                         │
│                │                   │                         │
│          [No Challenge]      [Challenge Submitted]           │
│                │                   │                         │
│                ▼                   ▼                         │
│         [Finalize Anchor]   [Verify Evidence]                │
│                │                   │                         │
│                │          ┌────────┴────────┐                │
│                │          │                 │                │
│                │    [Valid Proof]    [Invalid Proof]         │
│                │          │                 │                │
│                │          ▼                 ▼                │
│                │   [Slash Operator]  [Slash Challenger]      │
│                │          │                 │                │
│                ▼          ▼                 ▼                │
│              [State Finalized]                               │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### How It Works

#### 1. Operator Submits State Anchor

```rust
use ouro_dag::microchain::{ChallengeManager, StateAnchor};

let manager = ChallengeManager::new();

// Operator deposits stake
manager.deposit_operator_stake("microchain_abc".to_string(), 1_000_000_000); // 10 OURO

// Submit state anchor
let anchor_hash = manager.submit_anchor(
    "microchain_abc".to_string(),
    state_root,
    block_height,
    operator_signature,
    current_time,
).unwrap();

println!("⚓ State anchor submitted");
println!("   Block height: {}", block_height);
println!("   Challenge period: 7 days");
```

#### 2. User Challenges Invalid State

```rust
use ouro_dag::microchain::{ChallengeType, ChallengeEvidence};

// User deposits challenge bond
manager.deposit_challenge_bond("user1".to_string(), 10_000_000); // 0.1 OURO

// Prepare evidence
let evidence = ChallengeEvidence {
    previous_state_root: [0u8; 32],
    claimed_state_root: [1u8; 32],
    transactions: vec![/* ... */],
    merkle_proofs: vec![/* ... */],
    additional_data: vec![],
};

// Submit challenge
let challenge_id = manager.submit_challenge(
    anchor_hash,
    "user1".to_string(),
    ChallengeType::InvalidStateTransition,
    evidence,
    current_time + 86400, // 1 day later
).unwrap();

println!("⚠️  Challenge submitted: {}", challenge_id);
```

#### 3. System Verifies Challenge

```rust
// Verify challenge with microchain state
let result = manager.verify_challenge(
    &challenge_id,
    &microchain_state,
).unwrap();

if result {
    println!("⚡ Challenge ACCEPTED!");
    println!("   Operator slashed");
    println!("   Challenger rewarded");
} else {
    println!("❌ Challenge REJECTED!");
    println!("   Challenger bond slashed");
}
```

### Force Exit Mechanism

Users can force exit from a microchain if operators are unresponsive:

```rust
use ouro_dag::microchain::ForceExitRequest;

// Request force exit with merkle proof
let exit_id = manager.request_force_exit(
    "microchain_abc".to_string(),
    "user1".to_string(),
    50_000_000, // 0.5 OURO to withdraw
    nonce,
    merkle_proof,
    state_root,
    current_time,
).unwrap();

println!("🚪 Force exit requested: {}", exit_id);

// Process exit (verifies merkle proof)
let amount = manager.process_force_exit(&exit_id, current_time).unwrap();

println!("✅ Exit completed: {} OURO withdrawn", amount / 100_000_000);
```

### Challenge Types

| Type | Description | Slash Amount |
|------|-------------|--------------|
| `InvalidStateTransition` | State transition doesn't follow rules | 50% of operator stake |
| `UnauthorizedTransaction` | Transaction not properly signed | 50% of operator stake |
| `DoubleSpend` | Same nonce used twice | 50% of operator stake |
| `InvalidSignature` | Operator signature invalid | 50% of operator stake |
| `StateRootMismatch` | Computed root ≠ claimed root | 50% of operator stake |

### Security Parameters

| Parameter | Value | Purpose |
|-----------|-------|---------|
| Challenge Period | 7 days | Time to challenge anchor |
| Challenge Bond | 0.1 OURO | Prevent spam challenges |
| Operator Stake | 10+ OURO | Economic security |
| Slash Percentage | 50% | Penalty for fraud |
| Force Exit Delay | 1 day | Safety buffer |

---

## Fraud Detection Service

Automated monitoring system for detecting suspicious patterns.

### Architecture

```
┌────────────────────────────────────────────────────────────┐
│                  Fraud Detection Service                    │
├────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐     ┌──────────────┐     ┌────────────┐ │
│  │   Monitor    │────>│   Analyze    │────>│   Alert    │ │
│  │  Activities  │     │   Patterns   │     │  & Action  │ │
│  └──────────────┘     └──────────────┘     └────────────┘ │
│         │                     │                    │        │
│         │                     │                    │        │
│    ┌────▼─────┐          ┌───▼────┐          ┌────▼─────┐ │
│    │ Relayers │          │Pattern │          │Auto      │ │
│    │Operators │          │Matcher │          │Actions   │ │
│    │  Users   │          │Rules   │          │Blacklist │ │
│    └──────────┘          └────────┘          └──────────┘ │
│                                                              │
└────────────────────────────────────────────────────────────┘
```

### Setup

```rust
use ouro_dag::fraud_detection::{FraudDetectionService, AlertThresholds};

// Configure thresholds
let thresholds = AlertThresholds {
    max_failure_rate: 0.1,           // 10% failure triggers alert
    max_volume_per_hour: 100_000_000_000, // 1000 OURO per hour
    max_rapid_transactions: 100,      // 100 txs in short time
    min_anchor_frequency: 3600,       // 1 hour between anchors
};

let fraud_service = FraudDetectionService::new(thresholds);
```

### Monitoring Examples

#### Monitor Relay Activity

```rust
// Monitor each relay
let alert = fraud_service.monitor_relay(
    "relayer1".to_string(),
    5_000_000, // 0.05 OURO
    true,      // success
    current_time,
);

if let Some(alert) = alert {
    println!("🔴 ALERT: {}", alert.description);

    // Take automated action
    if let Some(action) = alert.auto_action {
        match action {
            AutoAction::PauseRelayer => {
                pause_relayer("relayer1");
            }
            AutoAction::SubmitFraudProof => {
                submit_fraud_proof_automatically(&alert);
            }
            _ => {}
        }
    }
}
```

#### Monitor Operator Activity

```rust
// Check if operator is anchoring regularly
let alert = fraud_service.monitor_operator(
    "operator_abc".to_string(),
    "microchain_abc".to_string(),
    last_anchor_time,
    current_time,
);

if let Some(alert) = alert {
    println!("🟠 Operator Alert: {}", alert.description);
    notify_admin(&alert);
}
```

#### Detect Transaction Patterns

```rust
// Monitor user transactions for double spend
let transactions = vec![
    (1, 1000),  // (nonce, timestamp)
    (2, 1001),
    (3, 1002),
];

let alert = fraud_service.monitor_transactions(
    "user1".to_string(),
    transactions,
    current_time,
);

if let Some(alert) = alert {
    println!("⚡ Fraud Detected: {}", alert.description);
    freeze_account("user1");
}
```

### Alert Management

#### Blacklist Malicious Entities

```rust
// Blacklist after repeated violations
fraud_service.blacklist_entity(
    "malicious_relayer".to_string(),
    "Multiple fraud attempts detected".to_string(),
    true, // permanent
    current_time,
);

// Check if blacklisted
if fraud_service.is_blacklisted("malicious_relayer") {
    return Err("Entity is blacklisted");
}
```

#### Generate Monitoring Report

```rust
let report = fraud_service.generate_report();
report.print();

// Output:
// ============================================================
//            FRAUD DETECTION MONITORING REPORT
// ============================================================
//
// 📊 Summary:
//    Total Alerts: 45
//    🔴 Critical: 3
//    🟠 High: 12
//
// 👥 Entities:
//    Total Monitored: 234
//    🚫 Blacklisted: 5
//
// ============================================================
```

#### Get Activity Statistics

```rust
let (total, successful, failed, volume) = fraud_service
    .get_activity_stats("relayer1")
    .unwrap();

println!("Relayer Statistics:");
println!("  Total relays: {}", total);
println!("  Success rate: {:.2}%", (successful as f64 / total as f64) * 100.0);
println!("  Total volume: {} OURO", volume / 100_000_000);
```

### Alert Types and Severities

| Alert Type | Severity | Auto Action |
|------------|----------|-------------|
| Blacklisted entity attempting relay | Critical | Pause relayer |
| High failure rate (>10%) | High | Increase monitoring |
| High value relay failed | Critical | Alert admin |
| Missing state anchor (>2h) | High | Alert admin |
| Double spend detected | Critical | Submit fraud proof |
| Abnormal volume | High | Increase monitoring |
| Rapid withdrawal pattern | Medium | Increase monitoring |

---

## Usage Examples

### Example 1: Complete Cross-Chain Transfer with Fraud Detection

```rust
use ouro_dag::cross_chain::{FraudProofManager, CrossChainMessage};
use ouro_dag::fraud_detection::FraudDetectionService;

async fn secure_cross_chain_transfer(
    from_subchain: &str,
    to_subchain: &str,
    sender: &str,
    recipient: &str,
    amount: u64,
) -> Result<(), String> {
    let fraud_proofs = FraudProofManager::new();
    let fraud_detection = FraudDetectionService::new(Default::default());

    // Step 1: Verify sender has sufficient balance
    let balance = get_balance(from_subchain, sender).await?;
    if balance < amount {
        return Err("Insufficient balance".to_string());
    }

    // Step 2: Lock funds on source chain
    lock_funds(from_subchain, sender, amount).await?;

    // Step 3: Create cross-chain message
    let message = CrossChainMessage {
        source_subchain: from_subchain.to_string(),
        destination_subchain: to_subchain.to_string(),
        sender: sender.to_string(),
        recipient: recipient.to_string(),
        amount,
        nonce: get_nonce(sender).await,
        timestamp: current_timestamp(),
    };

    // Step 4: Relayer submits message (optimistic)
    let message_hash = fraud_proofs.submit_relay(
        message.clone(),
        "relayer1".to_string(),
        Some(generate_merkle_proof(&message)),
        current_timestamp(),
    )?;

    // Step 5: Monitor relay for fraud
    let alert = fraud_detection.monitor_relay(
        "relayer1".to_string(),
        amount,
        true,
        current_timestamp(),
    );

    if let Some(alert) = alert {
        println!("⚠️  Alert triggered: {}", alert.description);
    }

    // Step 6: Execute transfer on destination (optimistic)
    execute_transfer(to_subchain, recipient, amount).await?;

    // Step 7: Challenge period (10 minutes)
    println!("✅ Transfer submitted. Challenge period: 10 minutes");

    // Background: Monitor for fraud proofs
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;

        // Confirm if no fraud proofs submitted
        fraud_proofs.confirm_relay(message_hash, current_timestamp() + 600)
            .expect("Failed to confirm relay");

        println!("✅ Transfer finalized!");
    });

    Ok(())
}
```

### Example 2: Microchain with Challenge Protection

```rust
use ouro_dag::microchain::{ChallengeManager, StateAnchor};
use ouro_dag::fraud_detection::FraudDetectionService;

async fn operate_microchain_with_protection(
    microchain_id: &str,
    operator: &str,
) -> Result<(), String> {
    let challenges = ChallengeManager::new();
    let fraud_detection = FraudDetectionService::new(Default::default());

    // Step 1: Deposit operator stake
    challenges.deposit_operator_stake(operator.to_string(), 1_000_000_000); // 10 OURO

    // Step 2: Process microchain transactions
    let transactions = process_microchain_transactions(microchain_id).await?;

    // Step 3: Compute new state root
    let new_state_root = compute_state_root(&transactions);

    // Step 4: Anchor state to mainchain
    let anchor_hash = challenges.submit_anchor(
        microchain_id.to_string(),
        new_state_root,
        get_block_height(microchain_id).await,
        sign_state_root(operator, &new_state_root),
        current_timestamp(),
    )?;

    // Step 5: Monitor operator activity
    fraud_detection.monitor_operator(
        operator.to_string(),
        microchain_id.to_string(),
        current_timestamp(),
        current_timestamp(),
    );

    println!("⚓ State anchored: {:?}", hex::encode(anchor_hash));
    println!("   Challenge period: 7 days");

    // Background: Wait for challenge period
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(7 * 86400)).await;

        // Finalize if no challenges
        challenges.finalize_anchor(anchor_hash, current_timestamp() + 7 * 86400)
            .expect("Failed to finalize anchor");

        println!("✅ State finalized!");
    });

    Ok(())
}
```

---

## Security Parameters

### Production Recommended Values

| Parameter | Testnet | Mainnet | Notes |
|-----------|---------|---------|-------|
| **Cross-Chain** |
| Relayer Bond | 10 OURO | 100 OURO | Higher for production |
| Challenge Period | 5 min | 10-30 min | Balance speed vs security |
| Fraud Reward | 50% | 50% | Incentivize fraud detection |
| **Microchain** |
| Operator Stake | 1 OURO | 10-100 OURO | Based on microchain value |
| Challenge Period | 1 day | 7-14 days | Longer for higher security |
| Challenge Bond | 0.01 OURO | 0.1-1 OURO | Prevent spam |
| Force Exit Delay | 1 hour | 1-3 days | Safety buffer |
| **Monitoring** |
| Max Failure Rate | 20% | 10% | Stricter for production |
| Max Volume/Hour | 10K OURO | 1K OURO | Adjust per use case |
| Alert Retention | 100 | 1000 | Storage vs visibility |

---

## Testing

### Run Fraud Proof Tests

```bash
cd ouro_dag

# Test cross-chain fraud proofs
cargo test --lib cross_chain::fraud_proofs::tests

# Test microchain challenges
cargo test --lib microchain::challenges::tests

# Test fraud detection
cargo test --lib fraud_detection::tests

# Run all fraud system tests
cargo test --lib fraud
```

### Integration Test Example

```rust
#[tokio::test]
async fn test_end_to_end_fraud_detection() {
    let fraud_proofs = FraudProofManager::new();
    let challenges = ChallengeManager::new();
    let monitoring = FraudDetectionService::new(Default::default());

    // Setup
    fraud_proofs.deposit_bond("relayer1".to_string(), 200_000_000);
    challenges.deposit_operator_stake("op1".to_string(), 1_000_000_000);

    // Test scenario: Fraudulent relay
    let message = CrossChainMessage {
        source_subchain: "us".to_string(),
        destination_subchain: "eu".to_string(),
        sender: "alice".to_string(),
        recipient: "bob".to_string(),
        amount: 10_000_000_000, // 100 OURO (Alice only has 1 OURO)
        nonce: 1,
        timestamp: 1000,
    };

    let hash = fraud_proofs.submit_relay(
        message.clone(),
        "relayer1".to_string(),
        None,
        1000,
    ).unwrap();

    // Monitor detects suspicious high value
    let alert = monitoring.monitor_relay("relayer1".to_string(), 10_000_000_000, true, 1000);
    assert!(alert.is_some());

    // Submit fraud proof
    fraud_proofs.submit_fraud_proof(
        hash,
        "challenger1".to_string(),
        FraudProofType::InsufficientBalance,
        vec![],
        1100,
    ).unwrap();

    // Verify and slash
    let mut source_state = HashMap::new();
    source_state.insert("alice".to_string(), 1_000_000); // Only 0.01 OURO

    let is_fraud = fraud_proofs.verify_and_slash(hash, &source_state, &HashMap::new()).unwrap();
    assert!(is_fraud);

    // Verify relayer was slashed
    assert_eq!(fraud_proofs.get_relay_status(hash), Some(RelayStatus::Slashed));
}
```

---

## Production Deployment

### 1. Configure Security Parameters

```toml
# config/fraud_detection.toml

[cross_chain]
relayer_bond = 10000000000  # 100 OURO
challenge_period_secs = 1800  # 30 minutes
fraud_reward_percentage = 50

[microchain]
min_operator_stake = 1000000000  # 10 OURO
challenge_period_secs = 604800  # 7 days
challenge_bond = 10000000  # 0.1 OURO
force_exit_delay_secs = 86400  # 1 day

[monitoring]
max_failure_rate = 0.10
max_volume_per_hour = 100000000000  # 1000 OURO
max_rapid_transactions = 100
alert_retention_count = 1000
```

### 2. Initialize Services

```rust
// src/main.rs
use ouro_dag::cross_chain::FraudProofManager;
use ouro_dag::microchain::ChallengeManager;
use ouro_dag::fraud_detection::{FraudDetectionService, AlertThresholds};

#[tokio::main]
async fn main() {
    // Load config
    let config = load_fraud_detection_config().await;

    // Initialize fraud proof managers
    let fraud_proofs = Arc::new(FraudProofManager::new());
    let challenges = Arc::new(ChallengeManager::new());
    let monitoring = Arc::new(FraudDetectionService::new(config.thresholds));

    // Start monitoring service
    let monitoring_clone = monitoring.clone();
    tokio::spawn(async move {
        loop {
            // Cleanup old alerts every hour
            monitoring_clone.cleanup_old_alerts();
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });

    // Start fraud proof verification service
    let fraud_proofs_clone = fraud_proofs.clone();
    tokio::spawn(async move {
        loop {
            // Check pending relays for fraud
            check_pending_relays(&fraud_proofs_clone).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    });

    // Start challenge verification service
    let challenges_clone = challenges.clone();
    tokio::spawn(async move {
        loop {
            // Check pending challenges
            check_pending_challenges(&challenges_clone).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        }
    });

    println!("✅ Fraud detection system initialized");

    // Start main application
    start_node(fraud_proofs, challenges, monitoring).await;
}
```

### 3. Monitor and Alert

```rust
// Setup webhooks for critical alerts
monitoring.add_alert_callback(|alert| {
    if alert.severity == AlertSeverity::Critical {
        send_webhook_notification("https://alerts.example.com/webhook", &alert);
        send_email_notification("admin@example.com", &alert);
    }
});

// Generate daily reports
tokio::spawn(async move {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;

        let report = monitoring.generate_report();
        report.print();

        save_report_to_database(&report).await;
    }
});
```

---

## Best Practices

### For Relayers

1. **Always deposit sufficient bond** before relaying messages
2. **Verify source chain state** before submitting relays
3. **Monitor your failure rate** and address issues promptly
4. **Keep bonds topped up** to avoid service interruption

### For Microchain Operators

1. **Deposit adequate stake** (10x expected microchain value)
2. **Anchor state regularly** (at least every hour)
3. **Maintain detailed logs** for dispute resolution
4. **Test force exit mechanism** before going live

### For Users

1. **Monitor your balances** on both mainchain and microchains
2. **Know the challenge periods** before trusting transfers
3. **Keep proof of your microchain transactions** for force exit
4. **Report suspicious activity** to earn fraud rewards

### For Administrators

1. **Set conservative thresholds** initially
2. **Monitor fraud detection reports** daily
3. **Respond to critical alerts** within 1 hour
4. **Review blacklist** weekly for false positives
5. **Adjust parameters** based on observed patterns

---

## Troubleshooting

### Common Issues

**Issue**: Relay rejected with "Insufficient bond"
- **Solution**: Deposit more bond using `deposit_bond()`

**Issue**: Challenge rejected after submission
- **Solution**: Verify evidence is correct and challenge period hasn't expired

**Issue**: Too many false positive alerts
- **Solution**: Increase thresholds in `AlertThresholds`

**Issue**: Force exit fails with "Invalid merkle proof"
- **Solution**: Ensure proof is for the most recent anchored state

---

## Conclusion

The Ouroboros fraud proof system provides comprehensive protection against:

✅ Fraudulent cross-chain relays
✅ Invalid microchain state transitions
✅ Malicious operators
✅ Double spend attempts
✅ Abnormal activity patterns

**Production Ready**: All components include automated testing, economic incentives, and monitoring.

**Next Steps**:
1. Configure security parameters for your deployment
2. Test thoroughly on testnet
3. Deploy monitoring infrastructure
4. Train operators on best practices
5. Launch with conservative parameters
6. Adjust based on observed behavior

---

## Resources

- **Code**: `ouro_dag/src/cross_chain/fraud_proofs.rs`
- **Code**: `ouro_dag/src/microchain/challenges.rs`
- **Code**: `ouro_dag/src/fraud_detection/mod.rs`
- **Tests**: Run `cargo test --lib fraud`
- **Examples**: See `examples/fraud_detection_demo.rs`

---

## License

MIT License - see LICENSE file for details
