use ouro_sdk::{Microchain, MicrochainConfig, ConsensusType, AnchorFrequency, Transaction};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ouroboros SDK - Advanced Microchain Example\n");

    // 1. Create a microchain with BFT consensus and custom settings
    println!("📦 Creating BFT microchain with custom settings...");
    let config = MicrochainConfig::new("HighSecurityDApp", "ouro1owner...")
        .with_consensus(ConsensusType::Bft { validator_count: 4 })
        .with_anchor_frequency(AnchorFrequency::EveryNBlocks(100))
        .with_block_time(10);

    let mut microchain = Microchain::create(config, "http://localhost:8001").await?;
    println!("✅ BFT Microchain ID: {}\n", microchain.id);

    // 2. Build custom transaction with data payload
    println!("🔨 Building custom transaction with data...");
    let mut tx = microchain.tx()
        .from("ouro1alice...")
        .to("ouro1smartcontract...")
        .amount(500)
        .data(json!({
            "method": "mint_nft",
            "params": {
                "token_id": "12345",
                "metadata": "ipfs://Qm..."
            }
        }))
        .build()?;

    // Sign transaction (in production, use actual keypair)
    // tx.sign_with_key("your_private_key_hex")?;

    println!("   Transaction ID: {}", tx.id);
    println!("   Data: {:?}\n", tx.data);

    // 3. Submit custom transaction
    println!("📤 Submitting custom transaction...");
    let tx_id = microchain.submit_tx(&tx).await?;
    println!("✅ Submitted: {}\n", tx_id);

    // 4. Query transaction history
    println!("📜 Fetching transaction history...");
    let txs = microchain.tx_history(0, 100).await?;
    println!("   Found {} transactions", txs.len());
    for (i, tx) in txs.iter().take(5).enumerate() {
        println!("   {}. {} -> {} ({})", i + 1, tx.from, tx.to, tx.amount);
    }
    println!();

    // 5. Query recent blocks
    println!("🧱 Fetching recent blocks...");
    let blocks = microchain.blocks(10).await?;
    println!("   Retrieved {} blocks", blocks.len());
    for block in &blocks {
        println!("   Block #{}: {} txs", block.height, block.tx_count);
    }
    println!();

    // 6. Connect to existing microchain
    println!("🔗 Connecting to existing microchain...");
    let existing = Microchain::connect(&microchain.id, "http://localhost:8001").await?;
    println!("✅ Connected to microchain: {}\n", existing.id);

    println!("🎉 Advanced example complete!");
    Ok(())
}
