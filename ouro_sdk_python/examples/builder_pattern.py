"""
Builder Pattern Example

Demonstrates the fluent API for creating microchains with different configurations
"""

from ouro_sdk import MicrochainBuilder, ConsensusType, AnchorFrequency


def main():
    print("🚀 Ouroboros SDK - Builder Pattern Example\n")

    # Example 1: Simple microchain
    print("📦 Creating simple microchain with builder...")
    simple = MicrochainBuilder("SimpleDApp", "ouro1owner...").node(
        "http://localhost:8001"
    ).build()

    print(f"✅ Created: {simple.id}\n")

    # Example 2: Gaming microchain with fast blocks
    print("🎮 Creating gaming microchain...")
    gaming = (
        MicrochainBuilder("GameFi", "ouro1gamedev...")
        .node("http://localhost:8001")
        .block_time(2)  # 2 second blocks for responsive gameplay
        .consensus(ConsensusType.SINGLE_VALIDATOR)  # Fast, centralized
        .anchor_frequency(AnchorFrequency.every_n_seconds(300))  # Anchor every 5 minutes
        .build()
    )

    print(f"✅ Gaming microchain: {gaming.id}")
    print("   Block time: 2 seconds")
    print("   Consensus: SingleValidator (fast)\n")

    # Example 3: DeFi microchain with high security
    print("💰 Creating DeFi microchain...")
    defi = (
        MicrochainBuilder("DeFiProtocol", "ouro1defi...")
        .node("http://localhost:8001")
        .block_time(10)  # 10 second blocks for stability
        .consensus(ConsensusType.BFT, 7)  # High security with 7 validators
        .anchor_frequency(AnchorFrequency.every_n_blocks(50))  # Frequent anchoring
        .build()
    )

    print(f"✅ DeFi microchain: {defi.id}")
    print("   Block time: 10 seconds")
    print("   Consensus: BFT with 7 validators")
    print("   Anchoring: Every 50 blocks\n")

    # Example 4: Manual anchoring microchain
    print("🔧 Creating microchain with manual anchoring...")
    manual = (
        MicrochainBuilder("ManualChain", "ouro1manual...")
        .node("http://localhost:8001")
        .anchor_frequency(AnchorFrequency.manual())
        .build()
    )

    print(f"✅ Manual microchain: {manual.id}")
    print("   Anchoring: Manual only\n")

    print("🎉 Builder pattern examples complete!")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"❌ Error: {e}")
        exit(1)
