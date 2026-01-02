use anyhow::{anyhow, Result};
use bech32::{Bech32, Hrp};
use bip39::{Language, Mnemonic};
use ed25519_dalek::{SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const WALLET_FILE: &str = "midgard_wallet.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub name: String,
    pub address: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    pub created_at: String,
}

impl Wallet {
    /// Generate a new wallet with BIP39 mnemonic
    pub fn generate(name: String) -> Result<(Self, String)> {
        // Generate 128 bits (16 bytes) of entropy for 12-word mnemonic
        let mut entropy = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| anyhow!("Failed to generate mnemonic: {}", e))?;
        let seed = mnemonic.to_seed("");

        // Use first 32 bytes as Ed25519 private key
        let private_key_bytes: [u8; SECRET_KEY_LENGTH] = seed[..SECRET_KEY_LENGTH]
            .try_into()
            .map_err(|_| anyhow!("Failed to generate private key"))?;

        let signing_key = SigningKey::from_bytes(&private_key_bytes);
        let verifying_key = signing_key.verifying_key();

        let address = Self::encode_address(&verifying_key)?;
        let public_key = hex::encode(verifying_key.to_bytes());
        let private_key = hex::encode(signing_key.to_keypair_bytes());

        let wallet = Wallet {
            name,
            address,
            public_key,
            private_key: Some(private_key),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok((wallet, mnemonic.to_string()))
    }

    /// Import wallet from mnemonic phrase
    pub fn from_mnemonic(mnemonic_phrase: &str, name: String) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .map_err(|e| anyhow!("Invalid mnemonic: {}", e))?;

        let seed = mnemonic.to_seed("");
        let private_key_bytes: [u8; SECRET_KEY_LENGTH] = seed[..SECRET_KEY_LENGTH]
            .try_into()
            .map_err(|_| anyhow!("Failed to generate private key"))?;

        let signing_key = SigningKey::from_bytes(&private_key_bytes);
        let verifying_key = signing_key.verifying_key();

        let address = Self::encode_address(&verifying_key)?;
        let public_key = hex::encode(verifying_key.to_bytes());
        let private_key = hex::encode(signing_key.to_keypair_bytes());

        Ok(Wallet {
            name,
            address,
            public_key,
            private_key: Some(private_key),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Import wallet from private key hex
    pub fn from_private_key(private_key_hex: &str, name: String) -> Result<Self> {
        let key_bytes = hex::decode(private_key_hex)
            .map_err(|_| anyhow!("Invalid hex private key"))?;

        if key_bytes.len() != 64 {
            return Err(anyhow!("Private key must be 64 bytes (keypair)"));
        }

        let signing_key = SigningKey::from_keypair_bytes(&key_bytes.try_into().unwrap())
            .map_err(|e| anyhow!("Invalid private key: {}", e))?;

        let verifying_key = signing_key.verifying_key();

        let address = Self::encode_address(&verifying_key)?;
        let public_key = hex::encode(verifying_key.to_bytes());

        Ok(Wallet {
            name,
            address,
            public_key,
            private_key: Some(private_key_hex.to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Encode public key to Bech32 address with "ouro" prefix
    fn encode_address(verifying_key: &VerifyingKey) -> Result<String> {
        let pubkey_bytes = verifying_key.to_bytes();

        // Try Bech32 encoding
        let hrp = Hrp::parse("ouro").map_err(|e| anyhow!("Invalid HRP: {}", e))?;

        match bech32::encode::<Bech32>(hrp, &pubkey_bytes) {
            Ok(addr) => Ok(addr),
            Err(_) => {
                // Fallback: ouro1 + first 20 bytes of pubkey in hex
                let short_addr = hex::encode(&pubkey_bytes[..20]);
                Ok(format!("ouro1{}", short_addr))
            }
        }
    }

    /// Get signing key from private key
    pub fn get_signing_key(&self) -> Result<SigningKey> {
        let private_key_hex = self.private_key
            .as_ref()
            .ok_or_else(|| anyhow!("No private key available"))?;

        let key_bytes = hex::decode(private_key_hex)
            .map_err(|_| anyhow!("Invalid hex private key"))?;

        if key_bytes.len() != 64 {
            return Err(anyhow!("Private key must be 64 bytes"));
        }

        SigningKey::from_keypair_bytes(&key_bytes.try_into().unwrap())
            .map_err(|e| anyhow!("Invalid signing key: {}", e))
    }

    /// Save wallet to file
    pub fn save(&self) -> Result<()> {
        let wallet_path = Self::get_wallet_path()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&wallet_path, json)?;
        Ok(())
    }

    /// Load wallet from file
    pub fn load() -> Result<Self> {
        let wallet_path = Self::get_wallet_path()?;
        if !wallet_path.exists() {
            return Err(anyhow!("No wallet found. Create one with 'midgard-wallet create'"));
        }

        let json = fs::read_to_string(&wallet_path)?;
        let wallet: Wallet = serde_json::from_str(&json)?;
        Ok(wallet)
    }

    /// Get wallet file path
    fn get_wallet_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        Ok(home.join(WALLET_FILE))
    }

    /// Check if wallet exists
    pub fn exists() -> bool {
        Self::get_wallet_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }
}
