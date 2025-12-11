// src/config.rs
// Configuration validation and secure key management

use std::env;
use log::{warn, info, error};

/// Validation result for configuration checks
pub struct ConfigValidation {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ConfigValidation {
    fn new() -> Self {
        Self {
            valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }

    fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
        self.valid = false;
    }

    pub fn print_summary(&self) {
        if !self.warnings.is_empty() {
            warn!("⚠️  Configuration Warnings:");
            for w in &self.warnings {
                warn!("   - {}", w);
            }
        }

        if !self.errors.is_empty() {
            error!("❌ Configuration Errors:");
            for e in &self.errors {
                error!("   - {}", e);
            }
        }

        if self.valid && self.warnings.is_empty() {
            info!("✅ Configuration validation passed");
        }
    }
}

/// Validate all critical configuration at startup
pub fn validate_config() -> ConfigValidation {
    let mut validation = ConfigValidation::new();

    info!("🔍 Validating configuration...");

    // Validate DATABASE_URL
    validate_database_url(&mut validation);

    // Validate API_KEYS (security)
    validate_api_keys(&mut validation);

    // Validate TLS configuration (optional but if set, must be valid)
    validate_tls_config(&mut validation);

    // Validate addresses
    validate_addresses(&mut validation);

    // Check for insecure configurations
    check_insecure_configs(&mut validation);

    validation
}

fn validate_database_url(validation: &mut ConfigValidation) {
    match env::var("DATABASE_URL") {
        Ok(url) if !url.is_empty() => {
            if url.contains("postgres://") || url.contains("postgresql://") {
                info!("✓ DATABASE_URL configured");

                // Warn if using default/insecure credentials
                if url.contains("postgres:postgres") {
                    validation.add_warning(
                        "DATABASE_URL uses default credentials (postgres:postgres) - INSECURE for production!".into()
                    );
                }
            } else {
                validation.add_error(
                    "DATABASE_URL must be a valid PostgreSQL connection string".into()
                );
            }
        }
        _ => {
            validation.add_warning(
                "DATABASE_URL not set - will use default (postgres://postgres:postgres@127.0.0.1:5432/postgres)".into()
            );
        }
    }
}

fn validate_api_keys(validation: &mut ConfigValidation) {
    match env::var("API_KEYS") {
        Ok(keys) if !keys.is_empty() => {
            let key_list: Vec<&str> = keys.split(',').map(|k| k.trim()).collect();

            if key_list.is_empty() {
                validation.add_error("API_KEYS is set but contains no valid keys".into());
                return;
            }

            info!("✓ API authentication enabled ({} key(s))", key_list.len());

            // Validate each key
            for (i, key) in key_list.iter().enumerate() {
                if key.len() < 32 {
                    validation.add_warning(
                        format!("API key #{} is too short ({} chars) - recommend at least 32 characters", i + 1, key.len())
                    );
                }

                // Check for obviously insecure keys
                if key.to_lowercase() == "password" ||
                   key.to_lowercase() == "secret" ||
                   *key == "12345" ||
                   *key == "test" {
                    validation.add_error(
                        format!("API key #{} is insecure (common/weak value) - MUST change for production!", i + 1)
                    );
                }
            }
        }
        _ => {
            // For lightweight nodes (RocksDB), API_KEYS is optional
            let storage_mode = env::var("STORAGE_MODE")
                .unwrap_or_else(|_| "postgres".into())
                .to_lowercase();

            if storage_mode.contains("rocks") {
                validation.add_warning(
                    "API_KEYS not set - Lightweight node will run without API authentication".into()
                );
            } else {
                validation.add_error(
                    "API_KEYS not set - Full validator node MUST have API authentication configured!".into()
                );
            }
        }
    }
}

fn validate_tls_config(validation: &mut ConfigValidation) {
    // API TLS validation
    let cert_path = env::var("TLS_CERT_PATH").ok();
    let key_path = env::var("TLS_KEY_PATH").ok();

    match (cert_path.clone(), key_path.clone()) {
        (Some(cert), Some(key)) => {
            info!("✓ API TLS configuration detected");

            // Check if files exist
            if !std::path::Path::new(&cert).exists() {
                validation.add_error(
                    format!("TLS_CERT_PATH points to non-existent file: {}", cert)
                );
            }

            if !std::path::Path::new(&key).exists() {
                validation.add_error(
                    format!("TLS_KEY_PATH points to non-existent file: {}", key)
                );
            }
        }
        (Some(_), None) => {
            validation.add_warning(
                "TLS_CERT_PATH set but TLS_KEY_PATH missing - TLS will be disabled".into()
            );
        }
        (None, Some(_)) => {
            validation.add_warning(
                "TLS_KEY_PATH set but TLS_CERT_PATH missing - TLS will be disabled".into()
            );
        }
        (None, None) => {
            validation.add_warning(
                "TLS not configured - API will run over HTTP (unencrypted)".into()
            );
        }
    }

    // P2P TLS validation (can use same certs as API or dedicated P2P certs)
    let p2p_cert = env::var("P2P_TLS_CERT_PATH").ok().or(cert_path);
    let p2p_key = env::var("P2P_TLS_KEY_PATH").ok().or(key_path);

    match (p2p_cert, p2p_key) {
        (Some(cert), Some(key)) => {
            info!("✓ P2P TLS configuration detected");

            // Check if files exist
            if !std::path::Path::new(&cert).exists() {
                validation.add_error(
                    format!("P2P TLS cert points to non-existent file: {}", cert)
                );
            }

            if !std::path::Path::new(&key).exists() {
                validation.add_error(
                    format!("P2P TLS key points to non-existent file: {}", key)
                );
            }
        }
        _ => {
            validation.add_warning(
                "P2P TLS not configured - peer connections will use plain TCP".into()
            );
        }
    }
}

fn validate_addresses(validation: &mut ConfigValidation) {
    // Validate API_ADDR
    let api_addr = env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".into());
    if let Err(_) = api_addr.parse::<std::net::SocketAddr>() {
        validation.add_error(
            format!("API_ADDR has invalid format: '{}' (expected IP:PORT)", api_addr)
        );
    }

    // Validate LISTEN_ADDR (P2P)
    let listen_addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:9000".into());
    if let Err(_) = listen_addr.parse::<std::net::SocketAddr>() {
        validation.add_error(
            format!("LISTEN_ADDR has invalid format: '{}' (expected IP:PORT)", listen_addr)
        );
    }
}

fn check_insecure_configs(validation: &mut ConfigValidation) {
    // Check if running in production-like environment
    let is_production = env::var("ENVIRONMENT")
        .map(|e| e.to_lowercase() == "production" || e.to_lowercase() == "prod")
        .unwrap_or(false);

    if is_production {
        info!("🏭 Production environment detected - enforcing stricter validation");

        // In production, certain things are errors not warnings
        if env::var("API_KEYS").is_err() {
            validation.add_error(
                "Production deployment MUST have API_KEYS configured!".into()
            );
        }

        if env::var("TLS_CERT_PATH").is_err() {
            validation.add_error(
                "Production deployment MUST use TLS/HTTPS!".into()
            );
        }
    }

    // Check rate limiting configuration
    if let Ok(max_req) = env::var("RATE_LIMIT_MAX_REQUESTS") {
        if let Ok(limit) = max_req.parse::<u32>() {
            if limit > 10000 {
                validation.add_warning(
                    format!("RATE_LIMIT_MAX_REQUESTS is very high ({}) - may not prevent DoS effectively", limit)
                );
            }
        }
    }
}

/// Generate a secure random API key (for admin use)
pub fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
