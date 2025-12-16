use ouro_sdk::{OuroClient, Transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ouroboros SDK - Client Usage Example\n");

    // Create a client to interact with Ouroboros node
    let client = OuroClient::new("http://localhost:8001");

    // 1. Check node health
    println!("🏥 Checking node health...");
    let healthy = client.health_check().await?;
    println!("   Node status: {}\n", if healthy { "✅ Healthy" } else { "❌ Unhealthy" });

    // 2. Get mainchain balance
    println!("💰 Checking mainchain balance...");
    let balance = client.get_balance("ouro1alice...").await?;
    println!("   Address: {}", balance.address);
    println!("   Balance: {} OURO", balance.balance);
    println!("   Pending: {} OURO\n", balance.pending);

    // 3. List all microchains
    println!("📋 Listing all microchains...");
    let microchains = client.list_microchains().await?;
    println!("   Found {} microchains:", microchains.len());
    for mc in microchains.iter().take(5) {
        println!("   - {}: {} (height: {})", mc.id, mc.name, mc.block_height);
    }
    println!();

    // 4. Get specific microchain state
    if let Some(mc) = microchains.first() {
        println!("🔍 Getting microchain state...");
        let state = client.get_microchain_state(&mc.id).await?;
        println!("   ID: {}", state.id);
        println!("   Name: {}", state.name);
        println!("   Owner: {}", state.owner);
        println!("   Block Height: {}", state.block_height);
        println!("   Total Transactions: {}", state.tx_count);
        if let Some(anchor) = state.last_anchor_height {
            println!("   Last Anchor: Block #{}", anchor);
        }
        println!();

        // 5. Check microchain balance
        println!("💎 Checking microchain balance...");
        let mc_balance = client.get_microchain_balance(&mc.id, "ouro1alice...").await?;
        println!("   Balance on {}: {} tokens\n", mc.name, mc_balance);
    }

    // 6. Submit mainchain transaction
    println!("📤 Submitting mainchain transaction...");
    let mut tx = Transaction::new("ouro1alice...", "ouro1bob...", 1000);
    // In production: tx.sign_with_key("private_key")?;
    tx.signature = "mock_signature".to_string();  // For demo only

    match client.submit_transaction(&tx).await {
        Ok(tx_id) => println!("✅ Transaction submitted: {}\n", tx_id),
        Err(e) => println!("❌ Transaction failed: {}\n", e),
    }

    // 7. Get transaction status
    println!("🔎 Checking transaction status...");
    match client.get_transaction_status(&tx.id).await {
        Ok(status) => println!("   Status: {:?}\n", status),
        Err(e) => println!("   Error: {}\n", e),
    }

    println!("🎉 Client usage example complete!");
    Ok(())
}
