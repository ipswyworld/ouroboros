use crate::error::{Result, SdkError};
use crate::transaction::Transaction;
use crate::types::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Main client for interacting with Ouroboros network
#[derive(Clone)]
pub struct OuroClient {
    base_url: String,
    client: Client,
}

impl OuroClient {
    /// Create a new client
    pub fn new(node_url: impl Into<String>) -> Self {
        Self {
            base_url: node_url.into().trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    /// Create a client with custom reqwest client
    pub fn with_client(node_url: impl Into<String>, client: Client) -> Self {
        Self {
            base_url: node_url.into().trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Get mainchain balance for address
    pub async fn get_balance(&self, address: &str) -> Result<Balance> {
        let url = format!("{}/balance/{}", self.base_url, address);
        let response: BalanceResponse = self.client.get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(Balance {
            address: address.to_string(),
            balance: response.balance,
            pending: response.pending.unwrap_or(0),
        })
    }

    /// Get microchain balance
    pub async fn get_microchain_balance(&self, microchain_id: &str, address: &str) -> Result<u64> {
        let url = format!("{}/microchain/{}/balance/{}", self.base_url, microchain_id, address);
        let response: MicrochainBalanceResponse = self.client.get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.balance)
    }

    /// Submit transaction to mainchain
    pub async fn submit_transaction(&self, tx: &Transaction) -> Result<String> {
        let url = format!("{}/tx/submit", self.base_url);
        let response: TxSubmitResponse = self.client.post(&url)
            .json(tx)
            .send()
            .await?
            .json()
            .await?;

        if response.success {
            Ok(response.tx_id)
        } else {
            Err(SdkError::TransactionFailed(
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, tx_id: &str) -> Result<TxStatus> {
        let url = format!("{}/tx/{}", self.base_url, tx_id);
        let response: TxStatusResponse = self.client.get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(match response.status.as_str() {
            "pending" => TxStatus::Pending,
            "confirmed" => TxStatus::Confirmed,
            "failed" => TxStatus::Failed,
            "anchored" => TxStatus::Anchored,
            _ => TxStatus::Pending,
        })
    }

    /// Create a new microchain
    pub async fn create_microchain(&self, config: &MicrochainConfig) -> Result<String> {
        let url = format!("{}/microchain/create", self.base_url);
        let response: CreateMicrochainResponse = self.client.post(&url)
            .json(config)
            .send()
            .await?
            .json()
            .await?;

        if response.success {
            Ok(response.microchain_id)
        } else {
            Err(SdkError::Other(
                response.message.unwrap_or_else(|| "Failed to create microchain".to_string())
            ))
        }
    }

    /// Get microchain state
    pub async fn get_microchain_state(&self, microchain_id: &str) -> Result<MicrochainState> {
        let url = format!("{}/microchain/{}/state", self.base_url, microchain_id);
        let state: MicrochainState = self.client.get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(state)
    }

    /// List all microchains
    pub async fn list_microchains(&self) -> Result<Vec<MicrochainState>> {
        let url = format!("{}/microchains", self.base_url);
        let response: ListMicrochainsResponse = self.client.get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.microchains)
    }

    /// Trigger manual anchor for a microchain
    pub async fn anchor_microchain(&self, microchain_id: &str) -> Result<String> {
        let url = format!("{}/microchain/{}/anchor", self.base_url, microchain_id);
        let response: AnchorResponse = self.client.post(&url)
            .send()
            .await?
            .json()
            .await?;

        if response.success {
            Ok(response.anchor_id)
        } else {
            Err(SdkError::AnchorFailed(
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// Check node health
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }
}

// Internal response types
#[derive(Deserialize)]
struct BalanceResponse {
    balance: u64,
    pending: Option<u64>,
}

#[derive(Deserialize)]
struct MicrochainBalanceResponse {
    balance: u64,
}

#[derive(Deserialize)]
struct TxSubmitResponse {
    success: bool,
    tx_id: String,
    message: Option<String>,
}

#[derive(Deserialize)]
struct TxStatusResponse {
    status: String,
}

#[derive(Deserialize)]
struct CreateMicrochainResponse {
    success: bool,
    microchain_id: String,
    message: Option<String>,
}

#[derive(Deserialize)]
struct ListMicrochainsResponse {
    microchains: Vec<MicrochainState>,
}

#[derive(Deserialize)]
struct AnchorResponse {
    success: bool,
    anchor_id: String,
    message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = OuroClient::new("http://localhost:8001");
        assert_eq!(client.base_url, "http://localhost:8001");
    }

    #[test]
    fn test_url_normalization() {
        let client = OuroClient::new("http://localhost:8001/");
        assert_eq!(client.base_url, "http://localhost:8001");
    }
}
