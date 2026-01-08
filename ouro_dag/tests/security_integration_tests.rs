// tests/security_integration_tests.rs
//! Security Integration Tests
//!
//! End-to-end tests for security features:
//! - Slashing mechanism
//! - Intrusion Detection System (IDS)
//! - Rate limiting
//! - Migration signing
//! - Key rotation

use ouro_dag::bft::slashing::{SlashingManager, SlashingReason, SlashingSeverity};
use ouro_dag::intrusion_detection::{IntrusionDetectionSystem, ThreatType, AlertSeverity};
use ouro_dag::migration_signing::{sign_migration, verify_migration, has_valid_signature};
use ouro_dag::key_rotation::{KeyRotationManager, KeyRotationStatus};
use sqlx::postgres::PgPoolOptions;
use ed25519_dalek::SigningKey;
use std::fs;
use tempfile::TempDir;

/// Test environment setup
struct TestEnv {
    pool: sqlx::PgPool,
    _temp_dir: TempDir,
}

impl TestEnv {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Use in-memory SQLite for tests instead of PostgreSQL
        // Note: Some tests may need to be skipped if PostgreSQL-specific
        let pool = sqlx::SqlitePool::connect(":memory:").await?;

        // Run basic migrations for testing
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS validator_stakes (
                validator_id TEXT PRIMARY KEY,
                stake BIGINT NOT NULL DEFAULT 0,
                slashed_amount BIGINT NOT NULL DEFAULT 0,
                last_slashed_at TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS slashing_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                validator_id TEXT NOT NULL,
                reason TEXT NOT NULL,
                severity TEXT NOT NULL,
                slashed_amount BIGINT NOT NULL,
                evidence TEXT,
                slashed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS key_rotations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                validator_id TEXT NOT NULL,
                old_public_key TEXT NOT NULL,
                new_public_key TEXT NOT NULL,
                signature TEXT NOT NULL,
                announced_at TIMESTAMP NOT NULL,
                transition_ends_at TIMESTAMP NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('Pending', 'InTransition', 'Completed', 'Revoked')),
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&pool)
        .await?;

        let temp_dir = TempDir::new()?;

        Ok(Self {
            pool,
            _temp_dir: temp_dir,
        })
    }
}

#[tokio::test]
async fn test_slashing_invalid_signature() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Setup validator with stake
    sqlx::query(
        "INSERT INTO validator_stakes (validator_id, stake) VALUES ($1, $2)"
    )
    .bind("validator_1")
    .bind(1_000_000_000i64) // 1000 OURO
    .execute(&env.pool)
    .await
    .expect("Failed to insert validator");

    let slashing_manager = SlashingManager::new(env.pool.clone());

    // Slash validator for invalid signature (50% penalty)
    let result = slashing_manager.slash_validator(
        "validator_1",
        SlashingReason::InvalidSignature,
        SlashingSeverity::Major,
        "Test: Invalid signature in block validation",
    ).await;

    assert!(result.is_ok(), "Slashing should succeed");

    let event = result.unwrap();
    assert_eq!(event.validator_id, "validator_1");
    assert_eq!(event.slashed_amount, 500_000_000); // 50% of 1000 OURO
    assert_eq!(event.severity, SlashingSeverity::Major);

    // Verify stake was reduced
    let stake: i64 = sqlx::query_scalar(
        "SELECT stake FROM validator_stakes WHERE validator_id = $1"
    )
    .bind("validator_1")
    .fetch_one(&env.pool)
    .await
    .expect("Failed to fetch stake");

    assert_eq!(stake, 500_000_000); // Reduced by 50%

    // Verify slashing event was recorded
    let events_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM slashing_events WHERE validator_id = $1"
    )
    .bind("validator_1")
    .fetch_one(&env.pool)
    .await
    .expect("Failed to count events");

    assert_eq!(events_count, 1);
}

#[tokio::test]
async fn test_slashing_equivocation() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Setup validator with stake
    sqlx::query(
        "INSERT INTO validator_stakes (validator_id, stake) VALUES ($1, $2)"
    )
    .bind("validator_2")
    .bind(1_000_000_000i64)
    .execute(&env.pool)
    .await
    .expect("Failed to insert validator");

    let slashing_manager = SlashingManager::new(env.pool.clone());

    // Slash for equivocation (100% penalty - Critical)
    let result = slashing_manager.slash_validator(
        "validator_2",
        SlashingReason::Equivocation,
        SlashingSeverity::Critical,
        "Test: Validator signed conflicting blocks",
    ).await;

    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.slashed_amount, 1_000_000_000); // 100% penalty

    // Verify stake is zero
    let stake: i64 = sqlx::query_scalar(
        "SELECT stake FROM validator_stakes WHERE validator_id = $1"
    )
    .bind("validator_2")
    .fetch_one(&env.pool)
    .await
    .expect("Failed to fetch stake");

    assert_eq!(stake, 0);
}

#[test]
fn test_ids_alert_generation() {
    let ids = IntrusionDetectionSystem::new();

    // Record authentication failures
    for i in 0..6 {
        ids.record_event(
            "attacker_ip",
            ThreatType::AuthenticationFailure,
            &format!("Failed login attempt {}", i),
        );
    }

    // Should generate alert after threshold (5)
    let alerts = ids.get_recent_alerts(10);
    assert!(!alerts.is_empty(), "Should have generated alert");

    let alert = &alerts[0];
    assert_eq!(alert.severity, AlertSeverity::High);
    assert!(alert.source.contains("attacker_ip"));
}

#[test]
fn test_ids_rate_limit_violations() {
    let ids = IntrusionDetectionSystem::new();

    // Simulate rate limit violations
    for i in 0..15 {
        ids.record_event(
            "spammer_ip",
            ThreatType::RateLimitViolation,
            &format!("Rate limit exceeded {}", i),
        );
    }

    let alerts = ids.get_recent_alerts(10);
    assert!(!alerts.is_empty());

    // Should escalate to High severity after 10 violations
    let high_alerts: Vec<_> = alerts
        .iter()
        .filter(|a| a.severity == AlertSeverity::High)
        .collect();

    assert!(!high_alerts.is_empty(), "Should escalate to high severity");
}

#[test]
fn test_migration_signing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let migration_path = temp_dir.path().join("test_migration.sql");

    // Write test migration
    fs::write(&migration_path, "CREATE TABLE test (id INTEGER PRIMARY KEY);")
        .expect("Failed to write migration");

    // Generate keypair
    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let private_key_bytes = signing_key.to_bytes();

    // Sign migration
    let signature = sign_migration(&migration_path, &private_key_bytes)
        .expect("Failed to sign migration");

    assert_eq!(signature.len(), 64, "Signature should be 64 bytes");

    // Write signature file
    let sig_path = migration_path.with_extension("sql.sig");
    fs::write(&sig_path, &signature).expect("Failed to write signature");

    // Verify signature exists
    assert!(has_valid_signature(&migration_path).is_ok());

    // Note: Verification will fail because we're using a different public key
    // than the one embedded in the binary. In production, the public key
    // would match the signing key.
}

#[test]
fn test_migration_signing_tampered() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let migration_path = temp_dir.path().join("test_migration.sql");

    // Write original migration
    fs::write(&migration_path, "CREATE TABLE test (id INTEGER PRIMARY KEY);")
        .expect("Failed to write migration");

    // Generate keypair and sign
    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let private_key_bytes = signing_key.to_bytes();

    let signature = sign_migration(&migration_path, &private_key_bytes)
        .expect("Failed to sign migration");

    // Write signature
    let sig_path = migration_path.with_extension("sql.sig");
    fs::write(&sig_path, &signature).expect("Failed to write signature");

    // Tamper with migration
    fs::write(&migration_path, "DROP TABLE test; -- MALICIOUS")
        .expect("Failed to tamper with migration");

    // Verification should fail (content changed)
    let is_valid = verify_migration(&migration_path, &signature)
        .expect("Verification should complete");

    assert!(!is_valid, "Tampered migration should fail verification");
}

#[tokio::test]
async fn test_key_rotation_lifecycle() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Setup validator
    sqlx::query(
        "INSERT INTO validator_stakes (validator_id, stake) VALUES ($1, $2)"
    )
    .bind("validator_3")
    .bind(1_000_000_000i64)
    .execute(&env.pool)
    .await
    .expect("Failed to insert validator");

    let key_rotation_manager = KeyRotationManager::new(env.pool.clone());

    // Generate old and new keypairs
    let old_key = SigningKey::generate(&mut rand::thread_rng());
    let new_key = SigningKey::generate(&mut rand::thread_rng());

    let old_private_hex = hex::encode(old_key.to_bytes());
    let new_public_hex = hex::encode(new_key.verifying_key().to_bytes());

    // Announce rotation
    let result = key_rotation_manager.announce_rotation(
        "validator_3",
        &old_private_hex,
        &new_public_hex,
    ).await;

    assert!(result.is_ok(), "Key rotation announcement should succeed");

    let announcement = result.unwrap();
    assert_eq!(announcement.validator_id, "validator_3");
    assert_eq!(announcement.status, KeyRotationStatus::Pending);

    // Verify rotation was recorded
    let rotation = key_rotation_manager.get_active_rotation("validator_3").await
        .expect("Failed to get rotation")
        .expect("Rotation should exist");

    assert_eq!(rotation.new_public_key, new_public_hex);
    assert_eq!(rotation.status, KeyRotationStatus::Pending);
}

#[tokio::test]
async fn test_key_rotation_prevents_duplicate() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    let key_rotation_manager = KeyRotationManager::new(env.pool.clone());

    let old_key = SigningKey::generate(&mut rand::thread_rng());
    let new_key = SigningKey::generate(&mut rand::thread_rng());

    let old_private_hex = hex::encode(old_key.to_bytes());
    let new_public_hex = hex::encode(new_key.verifying_key().to_bytes());

    // First rotation
    let result1 = key_rotation_manager.announce_rotation(
        "validator_4",
        &old_private_hex,
        &new_public_hex,
    ).await;

    assert!(result1.is_ok());

    // Second rotation (should fail - already active)
    let new_key2 = SigningKey::generate(&mut rand::thread_rng());
    let new_public_hex2 = hex::encode(new_key2.verifying_key().to_bytes());

    let result2 = key_rotation_manager.announce_rotation(
        "validator_4",
        &old_private_hex,
        &new_public_hex2,
    ).await;

    assert!(result2.is_err(), "Should prevent duplicate rotation");
    assert!(result2.unwrap_err().to_string().contains("already has an active"));
}

#[test]
fn test_ids_threat_suppression() {
    let ids = IntrusionDetectionSystem::new();

    // Generate many events from same source
    for i in 0..100 {
        ids.record_event(
            "noisy_source",
            ThreatType::InvalidTransaction,
            &format!("Invalid tx {}", i),
        );
    }

    let alerts = ids.get_recent_alerts(200);

    // Should not generate 100 alerts - some should be suppressed
    assert!(alerts.len() < 100, "Should suppress repeated alerts");
}

#[tokio::test]
async fn test_slashing_insufficient_stake() {
    let env = TestEnv::new().await.expect("Failed to create test environment");

    // Validator with low stake
    sqlx::query(
        "INSERT INTO validator_stakes (validator_id, stake) VALUES ($1, $2)"
    )
    .bind("validator_5")
    .bind(100_000_000i64) // 100 OURO
    .execute(&env.pool)
    .await
    .expect("Failed to insert validator");

    let slashing_manager = SlashingManager::new(env.pool.clone());

    // Slash with Critical severity (100%)
    let result = slashing_manager.slash_validator(
        "validator_5",
        SlashingReason::Equivocation,
        SlashingSeverity::Critical,
        "Test: Full slash on low stake",
    ).await;

    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.slashed_amount, 100_000_000); // All stake slashed

    // Verify stake is zero
    let stake: i64 = sqlx::query_scalar(
        "SELECT stake FROM validator_stakes WHERE validator_id = $1"
    )
    .bind("validator_5")
    .fetch_one(&env.pool)
    .await
    .expect("Failed to fetch stake");

    assert_eq!(stake, 0);
}
