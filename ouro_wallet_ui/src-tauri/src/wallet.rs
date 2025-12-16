use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use bip39::{Mnemonic, Language};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: String,
    pub public_key: String,
    #[serde(skip_serializing)]
    pub private_key: Option<String>,
    pub name: Option<String>,
    pub created_at: String,
}

impl Wallet {
    /// Generate a new wallet with mnemonic
    pub fn generate(name: Option<String>) -> (Self, String) {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words12, Language::English);
        let seed = mnemonic.phrase();

        let wallet = Self::from_mnemonic(seed, name).expect("Failed to create wallet from mnemonic");
        (wallet, seed.to_string())
    }

    /// Import wallet from mnemonic phrase
    pub fn from_mnemonic(mnemonic: &str, name: Option<String>) -> Result<Self, String> {
        let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| format!("Invalid mnemonic: {}", e))?;

        let seed = mnemonic.entropy();
        let mut seed_32 = [0u8; 32];
        seed_32.copy_from_slice(&seed[..32.min(seed.len())]);

        let secret = SecretKey::from_bytes(&seed_32)
            .map_err(|e| format!("Failed to create secret key: {}", e))?;
        let public = PublicKey::from(&secret);
        let keypair = Keypair { secret, public };

        let address = Self::encode_address(&keypair.public);

        Ok(Self {
            address,
            public_key: hex::encode(keypair.public.as_bytes()),
            private_key: Some(hex::encode(keypair.to_bytes())),
            name,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Import wallet from private key
    pub fn from_private_key(private_key_hex: &str, name: Option<String>) -> Result<Self, String> {
        let private_bytes = hex::decode(private_key_hex)
            .map_err(|e| format!("Invalid hex: {}", e))?;

        if private_bytes.len() != 64 {
            return Err("Private key must be 64 bytes".to_string());
        }

        let secret = SecretKey::from_bytes(&private_bytes[..32])
            .map_err(|e| format!("Invalid secret key: {}", e))?;
        let public = PublicKey::from(&secret);
        let keypair = Keypair { secret, public };

        let address = Self::encode_address(&keypair.public);

        Ok(Self {
            address,
            public_key: hex::encode(keypair.public.as_bytes()),
            private_key: Some(hex::encode(keypair.to_bytes())),
            name,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Encode public key to bech32 address
    fn encode_address(public_key: &PublicKey) -> String {
        use bech32::{ToBase32, Variant};
        let pub_bytes = public_key.as_bytes();
        bech32::encode("ouro", pub_bytes.to_base32(), Variant::Bech32)
            .unwrap_or_else(|_| format!("ouro1{}", hex::encode(&pub_bytes[..20])))
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<String, String> {
        let private_key = self.private_key.as_ref()
            .ok_or("No private key available")?;

        let private_bytes = hex::decode(private_key)
            .map_err(|e| format!("Invalid private key: {}", e))?;

        let keypair = Keypair::from_bytes(&private_bytes)
            .map_err(|e| format!("Invalid keypair: {}", e))?;

        let signature = keypair.sign(message);
        Ok(hex::encode(signature.to_bytes()))
    }

    /// Save wallet to file
    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;

        fs::write(path, json)
            .map_err(|e| format!("File write error: {}", e))?;

        Ok(())
    }

    /// Load wallet from file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let json = fs::read_to_string(path)
            .map_err(|e| format!("File read error: {}", e))?;

        serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization error: {}", e))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub public_key: String,
    pub name: Option<String>,
    pub created_at: String,
}

impl From<Wallet> for WalletInfo {
    fn from(wallet: Wallet) -> Self {
        Self {
            address: wallet.address,
            public_key: wallet.public_key,
            name: wallet.name,
            created_at: wallet.created_at,
        }
    }
}
