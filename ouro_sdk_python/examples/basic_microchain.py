"""
Basic Microchain Example

Demonstrates:
- Creating a microchain
- Checking state and balances
- Sending transactions
- Anchoring to mainchain
"""

from ouro_sdk import Microchain, MicrochainConfig


def main():
    print("🚀 Ouroboros SDK - Basic Microchain Example\n")

    # 1. Create a new microchain with default settings
    print("📦 Creating new microchain...")
    config = MicrochainConfig(
        name="MyDApp",
        owner="ouro1owner123...",
        block_time_secs=5,  # 5 second blocks
    )

    microchain = Microchain.create(config, "http://localhost:8001")
    print(f"✅ Microchain created with ID: {microchain.id}\n")

    # 2. Check microchain state
    print("🔍 Fetching microchain state...")
    state = microchain.state()
    print(f"   Name: {state.name}")
    print(f"   Owner: {state.owner}")
    print(f"   Block Height: {state.block_height}")
    print(f"   Total Transactions: {state.tx_count}\n")

    # 3. Check balance
    print("💰 Checking balance...")
    balance = microchain.balance("ouro1owner123...")
    print(f"   Balance: {balance} OURO\n")

    # 4. Transfer tokens
    print("💸 Sending transaction...")
    tx_id = microchain.transfer("ouro1owner123...", "ouro1recipient456...", 1000)
    print(f"✅ Transaction submitted: {tx_id}\n")

    # 5. Anchor to mainchain for security
    print("⚓ Anchoring to mainchain...")
    anchor_id = microchain.anchor()
    print(f"✅ Anchored with ID: {anchor_id}\n")

    print("🎉 Example complete!")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"❌ Error: {e}")
        exit(1)
