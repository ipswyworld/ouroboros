//! Fraud Detection System Demo
//!
//! Demonstrates the fraud detection system with realistic scenarios.
//!
//! Run with: cargo run --example fraud_detection_demo

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

// Import from ouro_dag crate
use ouro_dag::cross_chain::{
    CrossChainMessage, FraudProofManager, FraudProofType,
};
use ouro_dag::microchain::{
    ChallengeEvidence, ChallengeManager, ChallengeType,
};
use ouro_dag::fraud_detection::{
    FraudDetectionService, AlertThresholds, AlertSeverity,
};

fn main() {
    println!("\n{}", "=".repeat(80));
    println!("          OUROBOROS FRAUD DETECTION SYSTEM DEMO");
    println!("{}\n", "=".repeat(80));

    // Run demo scenarios
    demo_cross_chain_fraud_detection();
    demo_microchain_challenges();
    demo_fraud_monitoring();
    demo_admin_operations();

    println!("\n{}", "=".repeat(80));
    println!("                        DEMO COMPLETE");
    println!("{}\n", "=".repeat(80));
}

/// Demo 1: Cross-Chain Fraud Detection
fn demo_cross_chain_fraud_detection() {
    println!("\n📡 DEMO 1: Cross-Chain Fraud Detection");
    println!("{}\n", "-".repeat(80));

    let fraud_proofs = FraudProofManager::new();

    // Setup: Relayer deposits bond
    println!("Step 1: Relayer deposits bond");
    fraud_proofs.deposit_bond("honest_relayer".to_string(), 200_000_000);
    fraud_proofs.deposit_bond("dishonest_relayer".to_string(), 200_000_000);
    println!("✅ Bonds deposited: 2 OURO each\n");

    // Scenario A: Honest relay
    println!("Scenario A: Honest Relay");
    let honest_message = CrossChainMessage {
        source_subchain: "us_east".to_string(),
        destination_subchain: "eu_west".to_string(),
        sender: "alice".to_string(),
        recipient: "bob".to_string(),
        amount: 5_000_000, // 0.05 OURO
        nonce: 1,
        timestamp: 1000,
    };

    let hash1 = fraud_proofs.submit_relay(
        honest_message,
        "honest_relayer".to_string(),
        None,
        1000,
    ).unwrap();

    println!("  ✅ Relay submitted successfully");
    println!("  Message hash: {:?}", hex::encode(hash1));
    println!("  Challenge period: 10 minutes\n");

    // Wait for challenge period (simulated)
    println!("  ⏳ Waiting for challenge period...");
    thread::sleep(Duration::from_millis(100)); // Simulated

    // No fraud proofs submitted, confirm relay
    fraud_proofs.confirm_relay(hash1, 1000 + 601).unwrap();
    println!("  ✅ Relay confirmed! Honest relayer receives reward\n");

    // Scenario B: Fraudulent relay (insufficient balance)
    println!("Scenario B: Fraudulent Relay - Insufficient Balance");
    let fraud_message = CrossChainMessage {
        source_subchain: "us_east".to_string(),
        destination_subchain: "eu_west".to_string(),
        sender: "charlie".to_string(),
        recipient: "dave".to_string(),
        amount: 10_000_000_000, // 100 OURO (Charlie only has 1 OURO!)
        nonce: 1,
        timestamp: 2000,
    };

    let hash2 = fraud_proofs.submit_relay(
        fraud_message.clone(),
        "dishonest_relayer".to_string(),
        None,
        2000,
    ).unwrap();

    println!("  ⚠️  Suspicious relay submitted");
    println!("  Amount: 100 OURO (sender only has 1 OURO)");
    println!("  Message hash: {:?}\n", hex::encode(hash2));

    // Challenger detects fraud
    println!("  🕵️  Challenger detects fraud!");
    fraud_proofs.submit_fraud_proof(
        hash2,
        "watchdog_challenger".to_string(),
        FraudProofType::InsufficientBalance,
        vec![],
        2100,
    ).unwrap();
    println!("  ✅ Fraud proof submitted\n");

    // Verify fraud proof
    let mut source_state = HashMap::new();
    source_state.insert("charlie".to_string(), 1_000_000); // Only 0.01 OURO

    let is_fraud = fraud_proofs.verify_and_slash(
        hash2,
        &source_state,
        &HashMap::new(),
    ).unwrap();

    if is_fraud {
        println!("  ⚡ FRAUD PROVEN!");
        println!("  ❌ Dishonest relayer slashed: 1 OURO");
        println!("  ✅ Challenger rewarded: 0.5 OURO");
        println!("  🔒 Transaction reverted\n");
    }

    println!("📊 Summary:");
    println!("  Honest relayer bond: {} OURO", fraud_proofs.get_relayer_bond("honest_relayer") / 100_000_000);
    println!("  Dishonest relayer bond: {} OURO", fraud_proofs.get_relayer_bond("dishonest_relayer") / 100_000_000);
    println!("  Dishonest relayer slashed: {} OURO", fraud_proofs.get_slashed_amount("dishonest_relayer") / 100_000_000);
}

/// Demo 2: Microchain Challenge System
fn demo_microchain_challenges() {
    println!("\n⛓️  DEMO 2: Microchain Challenge System");
    println!("{}\n", "-".repeat(80));

    let challenges = ChallengeManager::new();

    // Setup: Operator deposits stake
    println!("Step 1: Operator deposits stake");
    challenges.deposit_operator_stake("microchain_abc".to_string(), 1_000_000_000);
    println!("✅ Stake deposited: 10 OURO\n");

    // Scenario A: Valid state anchor (no challenge)
    println!("Scenario A: Valid State Anchor");
    let valid_anchor = challenges.submit_anchor(
        "microchain_abc".to_string(),
        [1u8; 32],
        100,
        vec![0u8; 64],
        1000,
    ).unwrap();

    println!("  ⚓ State anchor submitted");
    println!("  Block height: 100");
    println!("  Challenge period: 7 days\n");

    thread::sleep(Duration::from_millis(50));

    // No challenges, finalize
    challenges.finalize_anchor(valid_anchor, 1000 + 604_801).unwrap();
    println!("  ✅ Anchor finalized (no challenges)\n");

    // Scenario B: Invalid state anchor (challenged)
    println!("Scenario B: Invalid State Anchor - Challenged");

    // User deposits challenge bond
    challenges.deposit_challenge_bond("concerned_user".to_string(), 20_000_000);
    println!("  💰 User deposits challenge bond: 0.2 OURO\n");

    let bad_anchor = challenges.submit_anchor(
        "microchain_abc".to_string(),
        [2u8; 32],
        200,
        vec![0u8; 64],
        2000,
    ).unwrap();

    println!("  ⚓ State anchor submitted (contains fraud)");
    println!("  Block height: 200\n");

    // User submits challenge
    let evidence = ChallengeEvidence {
        previous_state_root: [1u8; 32],
        claimed_state_root: [2u8; 32],
        transactions: vec![],
        merkle_proofs: vec![],
        additional_data: vec![],
    };

    let challenge_id = challenges.submit_challenge(
        bad_anchor,
        "concerned_user".to_string(),
        ChallengeType::StateRootMismatch,
        evidence,
        2500,
    ).unwrap();

    println!("  ⚠️  Challenge submitted: {}", challenge_id);
    println!("  Type: StateRootMismatch\n");

    // Verify challenge
    let is_valid = challenges.verify_challenge(
        &challenge_id,
        &HashMap::new(),
    ).unwrap();

    if is_valid {
        println!("  ⚡ CHALLENGE ACCEPTED!");
        println!("  ❌ Operator slashed: 5 OURO (50% of stake)");
        println!("  ✅ Challenger rewarded: 5 OURO");
        println!("  🔒 Invalid anchor rejected\n");
    }

    // Scenario C: Force Exit
    println!("Scenario C: Force Exit from Microchain");

    let exit_id = challenges.request_force_exit(
        "microchain_abc".to_string(),
        "trapped_user".to_string(),
        50_000_000, // 0.5 OURO
        1,
        vec![[1u8; 32], [2u8; 32]], // Merkle proof
        [3u8; 32], // State root
        3000,
    ).unwrap();

    println!("  🚪 Force exit requested");
    println!("  User: trapped_user");
    println!("  Amount: 0.5 OURO\n");

    let amount = challenges.process_force_exit(&exit_id, 3100).unwrap();

    println!("  ✅ Force exit completed!");
    println!("  Amount withdrawn: {} OURO", amount / 100_000_000);
}

/// Demo 3: Fraud Monitoring Service
fn demo_fraud_monitoring() {
    println!("\n🔍 DEMO 3: Fraud Monitoring Service");
    println!("{}\n", "-".repeat(80));

    let thresholds = AlertThresholds {
        max_failure_rate: 0.2, // 20% for demo
        max_volume_per_hour: 100_000_000_000,
        max_rapid_transactions: 5, // Low threshold for demo
        min_anchor_frequency: 3600,
    };

    let monitoring = FraudDetectionService::new(thresholds);

    // Scenario A: Normal activity
    println!("Scenario A: Normal Activity");
    for i in 1..=5 {
        let alert = monitoring.monitor_relay(
            "good_relayer".to_string(),
            1_000_000,
            true,
            1000 + i,
        );

        if alert.is_none() {
            println!("  ✅ Relay {} succeeded - no alerts", i);
        }
    }
    println!();

    // Scenario B: High failure rate
    println!("Scenario B: High Failure Rate Detection");
    for i in 1..=10 {
        let success = i % 4 != 0; // 75% failure rate
        let alert = monitoring.monitor_relay(
            "bad_relayer".to_string(),
            1_000_000,
            success,
            2000 + i,
        );

        if let Some(alert) = alert {
            println!("  🔴 ALERT: {}", alert.description);
            println!("     Severity: {:?}", alert.severity);
            println!("     Auto-action: {:?}\n", alert.auto_action);
            break;
        }
    }

    // Scenario C: Double spend detection
    println!("Scenario C: Double Spend Detection");
    let transactions = vec![
        (1, 3000),
        (2, 3001),
        (1, 3002), // Duplicate nonce!
    ];

    let alert = monitoring.monitor_transactions(
        "malicious_user".to_string(),
        transactions,
        3003,
    );

    if let Some(alert) = alert {
        println!("  ⚡ CRITICAL ALERT: {}", alert.description);
        println!("     Type: {:?}", alert.alert_type);
        println!("     Auto-action: Submit fraud proof\n");
    }

    // Scenario D: Blacklist system
    println!("Scenario D: Blacklist Management");
    monitoring.blacklist_entity(
        "repeat_offender".to_string(),
        "Multiple fraud attempts".to_string(),
        true, // Permanent
        4000,
    );

    println!("  🚫 Entity blacklisted: repeat_offender");
    println!("  Reason: Multiple fraud attempts");
    println!("  Permanent: Yes\n");

    let alert = monitoring.monitor_relay(
        "repeat_offender".to_string(),
        1_000_000,
        true,
        4100,
    );

    if let Some(alert) = alert {
        println!("  ⚡ ALERT: Blacklisted entity attempted action");
        println!("     Severity: {:?}", alert.severity);
        println!("     Auto-blocked: ✅\n");
    }

    // Generate report
    println!("📊 Monitoring Report:");
    let report = monitoring.generate_report();
    report.print();
}

/// Demo 4: Admin Operations
fn demo_admin_operations() {
    println!("\n👨‍💼 DEMO 4: Admin Operations");
    println!("{}\n", "-".repeat(80));

    let monitoring = FraudDetectionService::new(AlertThresholds::default());

    // Simulate some activity
    monitoring.monitor_relay("relayer_1".to_string(), 10_000_000, true, 1000);
    monitoring.monitor_relay("relayer_1".to_string(), 20_000_000, true, 1001);
    monitoring.monitor_relay("relayer_2".to_string(), 50_000_000, false, 1002);

    println!("Admin Task 1: Check entity statistics");
    if let Some((total, successful, failed, volume)) = monitoring.get_activity_stats("relayer_1") {
        println!("  Entity: relayer_1");
        println!("  Total relays: {}", total);
        println!("  Success rate: {:.1}%", (successful as f64 / total as f64) * 100.0);
        println!("  Total volume: {} OURO\n", volume / 100_000_000);
    }

    println!("Admin Task 2: Review recent alerts");
    let alerts = monitoring.get_recent_alerts(10);
    println!("  Recent alerts: {}", alerts.len());
    for alert in alerts.iter().take(3) {
        println!("    - {} ({})", alert.description, format!("{:?}", alert.severity));
    }
    println!();

    println!("Admin Task 3: Check critical alerts");
    let critical = monitoring.get_alerts_by_severity(AlertSeverity::Critical);
    println!("  Critical alerts: {}\n", critical.len());

    println!("Admin Task 4: Verify blacklist");
    monitoring.blacklist_entity(
        "test_entity".to_string(),
        "Test".to_string(),
        false,
        2000,
    );
    let is_blacklisted = monitoring.is_blacklisted("test_entity");
    println!("  test_entity blacklisted: {}\n", is_blacklisted);

    println!("Admin Task 5: Generate daily report");
    let report = monitoring.generate_report();
    println!("  Report generated:");
    println!("    Total alerts: {}", report.total_alerts);
    println!("    Critical: {}", report.critical_alerts);
    println!("    Monitored entities: {}", report.total_entities);
}
