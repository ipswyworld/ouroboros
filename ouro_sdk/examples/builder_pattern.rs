use ouro_sdk::{MicrochainBuilder, ConsensusType, AnchorFrequency};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ouroboros SDK - Builder Pattern Example\n");

    // The builder pattern provides a fluent API for creating microchains

    // Example 1: Simple microchain
    println!("📦 Creating simple microchain with builder...");
    let simple = MicrochainBuilder::new("SimpleDApp", "ouro1owner...")
        .node("http://localhost:8001")
        .build()
        .await?;

    println!("✅ Created: {}\n", simple.id);

    // Example 2: Gaming microchain with fast blocks
    println!("🎮 Creating gaming microchain...");
    let gaming = MicrochainBuilder::new("GameFi", "ouro1gamedev...")
        .node("http://localhost:8001")
        .block_time(2)  // 2 second blocks for responsive gameplay
        .consensus(ConsensusType::SingleValidator)  // Fast, centralized
        .anchor_frequency(AnchorFrequency::EveryNSeconds(300))  // Anchor every 5 minutes
        .build()
        .await?;

    println!("✅ Gaming microchain: {}", gaming.id);
    println!("   Block time: 2 seconds");
    println!("   Consensus: SingleValidator (fast)\n");

    // Example 3: DeFi microchain with high security
    println!("💰 Creating DeFi microchain...");
    let defi = MicrochainBuilder::new("DeFiProtocol", "ouro1defi...")
        .node("http://localhost:8001")
        .block_time(10)  // 10 second blocks for stability
        .consensus(ConsensusType::Bft { validator_count: 7 })  // High security
        .anchor_frequency(AnchorFrequency::EveryNBlocks(50))  // Frequent anchoring
        .build()
        .await?;

    println!("✅ DeFi microchain: {}", defi.id);
    println!("   Block time: 10 seconds");
    println!("   Consensus: BFT with 7 validators");
    println!("   Anchoring: Every 50 blocks\n");

    // Example 4: Manual anchoring microchain
    println!("🔧 Creating microchain with manual anchoring...");
    let manual = MicrochainBuilder::new("ManualChain", "ouro1manual...")
        .node("http://localhost:8001")
        .anchor_frequency(AnchorFrequency::Manual)
        .build()
        .await?;

    println!("✅ Manual microchain: {}", manual.id);
    println!("   Anchoring: Manual only\n");

    println!("🎉 Builder pattern examples complete!");
    Ok(())
}
