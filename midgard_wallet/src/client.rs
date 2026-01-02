use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

const DEFAULT_API_URL: &str = "http://localhost:8001";
const DEFAULT_API_KEY: &str = "default_api_key";

#[derive(Debug, Deserialize)]
pub struct BalanceResponse {
    pub balance: u64,
}

#[derive(Debug, Deserialize)]
pub struct TransactionResponse {
    pub tx_id: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub block_height: u64,
}

#[derive(Debug, Deserialize)]
pub struct NonceResponse {
    pub nonce: u64,
}

pub struct OuroClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl OuroClient {
    /// Create new client with custom URL
    pub fn new(url: Option<String>) -> Self {
        OuroClient {
            client: Client::new(),
            base_url: url.unwrap_or_else(|| DEFAULT_API_URL.to_string()),
            api_key: DEFAULT_API_KEY.to_string(),
        }
    }

    /// Get balance for an address
    pub fn get_balance(&self, address: &str) -> Result<u64> {
        let url = format!("{}/ouro/balance/{}", self.base_url, address);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch balance: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, error_text));
        }

        let balance_response: BalanceResponse = response
            .json()
            .map_err(|e| anyhow!("Failed to parse balance response: {}", e))?;

        Ok(balance_response.balance)
    }

    /// Submit a transaction
    pub fn submit_transaction(&self, tx_json: Value) -> Result<String> {
        let url = format!("{}/tx/submit", self.base_url);

        let response = self.client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&tx_json)
            .send()
            .map_err(|e| anyhow!("Failed to submit transaction: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!("Transaction submission failed {}: {}", status, error_text));
        }

        let tx_response: TransactionResponse = response
            .json()
            .map_err(|e| anyhow!("Failed to parse transaction response: {}", e))?;

        Ok(tx_response.tx_id)
    }

    /// Get current block height
    pub fn get_status(&self) -> Result<u64> {
        let url = format!("{}/status", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch status: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get status: {}", response.status()));
        }

        let status_response: StatusResponse = response
            .json()
            .map_err(|e| anyhow!("Failed to parse status response: {}", e))?;

        Ok(status_response.block_height)
    }

    /// Health check
    pub fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| anyhow!("Failed to connect to node: {}", e))?;

        Ok(response.status().is_success())
    }

    /// Get nonce for an address
    pub fn get_nonce(&self, address: &str) -> Result<u64> {
        let url = format!("{}/ouro/nonce/{}", self.base_url, address);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch nonce: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, error_text));
        }

        let nonce_response: NonceResponse = response
            .json()
            .map_err(|e| anyhow!("Failed to parse nonce response: {}", e))?;

        Ok(nonce_response.nonce)
    }
}
