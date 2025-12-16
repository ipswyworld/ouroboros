use ouro_sdk::{Microchain, MicrochainConfig, ConsensusType, AnchorFrequency, Transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ouroboros SDK - Basic Microchain Example\n");

    // 1. Create a new microchain with default settings
    println!("📦 Creating new microchain...");
    let config = MicrochainConfig::new("MyDApp", "ouro1owner123...")
        .with_block_time(5);  // 5 second blocks

    let mut microchain = Microchain::create(config, "http://localhost:8001").await?;
    println!("✅ Microchain created with ID: {}\n", microchain.id);

    // 2. Check microchain state
    println!("🔍 Fetching microchain state...");
    let state = microchain.state().await?;
    println!("   Name: {}", state.name);
    println!("   Owner: {}", state.owner);
    println!("   Block Height: {}", state.block_height);
    println!("   Total Transactions: {}\n", state.tx_count);

    // 3. Check balance
    println!("💰 Checking balance...");
    let balance = microchain.balance("ouro1owner123...").await?;
    println!("   Balance: {} OURO\n", balance);

    // 4. Transfer tokens
    println!("💸 Sending transaction...");
    let tx_id = microchain.transfer(
        "ouro1owner123...",
        "ouro1recipient456...",
        1000
    ).await?;
    println!("✅ Transaction submitted: {}\n", tx_id);

    // 5. Anchor to mainchain for security
    println!("⚓ Anchoring to mainchain...");
    let anchor_id = microchain.anchor().await?;
    println!("✅ Anchored with ID: {}\n", anchor_id);

    println!("🎉 Example complete!");
    Ok(())
}
