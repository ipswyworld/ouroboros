use crate::wallet::{Wallet, WalletInfo};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;

// Response types
#[derive(Debug, Serialize)]
pub struct CreateWalletResponse {
    pub wallet: WalletInfo,
    pub mnemonic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance: u64,
    pub pending: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MicrochainInfo {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub block_height: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: String,
    pub status: String,
}

// Helper to get wallet path
fn get_wallet_path(app: &AppHandle) -> PathBuf {
    app.path_resolver()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("wallet.json")
}

/// Create a new wallet
#[tauri::command]
pub fn create_wallet(
    name: Option<String>,
    app: AppHandle,
) -> Result<CreateWalletResponse, String> {
    let (wallet, mnemonic) = Wallet::generate(name);
    let wallet_path = get_wallet_path(&app);

    wallet.save(&wallet_path)?;

    Ok(CreateWalletResponse {
        wallet: wallet.into(),
        mnemonic,
    })
}

/// Import wallet from mnemonic
#[tauri::command]
pub fn import_wallet(
    mnemonic: String,
    name: Option<String>,
    app: AppHandle,
) -> Result<WalletInfo, String> {
    let wallet = Wallet::from_mnemonic(&mnemonic, name)?;
    let wallet_path = get_wallet_path(&app);

    wallet.save(&wallet_path)?;

    Ok(wallet.into())
}

/// Import wallet from private key
#[tauri::command]
pub fn import_from_key(
    private_key: String,
    name: Option<String>,
    app: AppHandle,
) -> Result<WalletInfo, String> {
    let wallet = Wallet::from_private_key(&private_key, name)?;
    let wallet_path = get_wallet_path(&app);

    wallet.save(&wallet_path)?;

    Ok(wallet.into())
}

/// Get wallet information
#[tauri::command]
pub fn get_wallet_info(app: AppHandle) -> Result<WalletInfo, String> {
    let wallet_path = get_wallet_path(&app);

    if !wallet_path.exists() {
        return Err("No wallet found. Please create or import a wallet.".to_string());
    }

    let wallet = Wallet::load(&wallet_path)?;
    Ok(wallet.into())
}

/// Export mnemonic (requires wallet to have private key)
#[tauri::command]
pub fn export_mnemonic(app: AppHandle) -> Result<String, String> {
    let wallet_path = get_wallet_path(&app);
    let wallet = Wallet::load(&wallet_path)?;

    // For security, in production you'd want to verify password here
    wallet
        .private_key
        .ok_or("Wallet does not have private key stored".to_string())
}

/// Get balance from node
#[tauri::command]
pub async fn get_balance(node_url: String, app: AppHandle) -> Result<BalanceResponse, String> {
    let wallet_path = get_wallet_path(&app);
    let wallet = Wallet::load(&wallet_path)?;

    let client = reqwest::Client::new();
    let url = format!("{}/balance/{}", node_url.trim_end_matches('/'), wallet.address);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON error: {}", e))?;

    Ok(BalanceResponse {
        balance: data["balance"].as_u64().unwrap_or(0),
        pending: data["pending"].as_u64().unwrap_or(0),
    })
}

/// Get microchain balance
#[tauri::command]
pub async fn get_microchain_balance(
    node_url: String,
    microchain_id: String,
    app: AppHandle,
) -> Result<u64, String> {
    let wallet_path = get_wallet_path(&app);
    let wallet = Wallet::load(&wallet_path)?;

    let client = reqwest::Client::new();
    let url = format!(
        "{}/microchain/{}/balance/{}",
        node_url.trim_end_matches('/'),
        microchain_id,
        wallet.address
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON error: {}", e))?;

    Ok(data["balance"].as_u64().unwrap_or(0))
}

/// Send transaction
#[tauri::command]
pub async fn send_transaction(
    node_url: String,
    to: String,
    amount: u64,
    app: AppHandle,
) -> Result<String, String> {
    let wallet_path = get_wallet_path(&app);
    let wallet = Wallet::load(&wallet_path)?;

    // Create transaction
    let tx_id = uuid::Uuid::new_v4().to_string();
    let nonce = 1; // In production, get from node
    let message = format!("{}:{}:{}:{}:{}", tx_id, wallet.address, to, amount, nonce);
    let signature = wallet.sign(message.as_bytes())?;

    let tx_data = serde_json::json!({
        "id": tx_id,
        "from": wallet.address,
        "to": to,
        "amount": amount,
        "nonce": nonce,
        "signature": signature,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let client = reqwest::Client::new();
    let url = format!("{}/tx/submit", node_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .json(&tx_data)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON error: {}", e))?;

    if data["success"].as_bool().unwrap_or(false) {
        Ok(data["tx_id"].as_str().unwrap_or(&tx_id).to_string())
    } else {
        Err(data["message"]
            .as_str()
            .unwrap_or("Transaction failed")
            .to_string())
    }
}

/// Send microchain transaction
#[tauri::command]
pub async fn send_microchain_transaction(
    node_url: String,
    microchain_id: String,
    to: String,
    amount: u64,
    app: AppHandle,
) -> Result<String, String> {
    let wallet_path = get_wallet_path(&app);
    let wallet = Wallet::load(&wallet_path)?;

    let tx_id = uuid::Uuid::new_v4().to_string();
    let nonce = 1;
    let message = format!("{}:{}:{}:{}:{}", tx_id, wallet.address, to, amount, nonce);
    let signature = wallet.sign(message.as_bytes())?;

    let tx_data = serde_json::json!({
        "id": tx_id,
        "from": wallet.address,
        "to": to,
        "amount": amount,
        "nonce": nonce,
        "signature": signature,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let client = reqwest::Client::new();
    let url = format!(
        "{}/microchain/{}/tx",
        node_url.trim_end_matches('/'),
        microchain_id
    );

    let response = client
        .post(&url)
        .json(&tx_data)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON error: {}", e))?;

    if data["success"].as_bool().unwrap_or(false) {
        Ok(data["tx_id"].as_str().unwrap_or(&tx_id).to_string())
    } else {
        Err(data["message"]
            .as_str()
            .unwrap_or("Transaction failed")
            .to_string())
    }
}

/// List microchains
#[tauri::command]
pub async fn list_microchains(node_url: String) -> Result<Vec<MicrochainInfo>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/microchains", node_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON error: {}", e))?;

    let microchains: Vec<MicrochainInfo> = data["microchains"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|mc| MicrochainInfo {
            id: mc["id"].as_str().unwrap_or("").to_string(),
            name: mc["name"].as_str().unwrap_or("").to_string(),
            owner: mc["owner"].as_str().unwrap_or("").to_string(),
            block_height: mc["blockHeight"].as_u64().unwrap_or(0),
        })
        .collect();

    Ok(microchains)
}

/// Link wallet to node
#[tauri::command]
pub async fn link_to_node(node_url: String, app: AppHandle) -> Result<String, String> {
    let wallet_path = get_wallet_path(&app);
    let wallet = Wallet::load(&wallet_path)?;

    // Sign linking message
    let message = format!("Link wallet {} to node", wallet.address);
    let signature = wallet.sign(message.as_bytes())?;

    let link_data = serde_json::json!({
        "wallet_address": wallet.address,
        "signature": signature,
    });

    let client = reqwest::Client::new();
    let url = format!("{}/wallet/link", node_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .json(&link_data)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON error: {}", e))?;

    if data["success"].as_bool().unwrap_or(false) {
        Ok("Wallet linked successfully".to_string())
    } else {
        Err(data["message"]
            .as_str()
            .unwrap_or("Linking failed")
            .to_string())
    }
}

/// Get transaction history
#[tauri::command]
pub async fn get_transaction_history(
    node_url: String,
    app: AppHandle,
) -> Result<Vec<TransactionRecord>, String> {
    let wallet_path = get_wallet_path(&app);
    let wallet = Wallet::load(&wallet_path)?;

    let client = reqwest::Client::new();
    let url = format!(
        "{}/transactions/{}",
        node_url.trim_end_matches('/'),
        wallet.address
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("JSON error: {}", e))?;

    let transactions: Vec<TransactionRecord> = data["transactions"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|tx| TransactionRecord {
            id: tx["id"].as_str().unwrap_or("").to_string(),
            from: tx["from"].as_str().unwrap_or("").to_string(),
            to: tx["to"].as_str().unwrap_or("").to_string(),
            amount: tx["amount"].as_u64().unwrap_or(0),
            timestamp: tx["timestamp"].as_str().unwrap_or("").to_string(),
            status: tx["status"].as_str().unwrap_or("pending").to_string(),
        })
        .collect();

    Ok(transactions)
}
