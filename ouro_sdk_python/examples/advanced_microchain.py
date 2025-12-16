"""
Advanced Microchain Example

Demonstrates:
- BFT consensus configuration
- Custom transactions with data payloads
- Transaction history queries
- Block queries
- Connecting to existing microchains
"""

from ouro_sdk import Microchain, MicrochainConfig, ConsensusType


def main():
    print("🚀 Ouroboros SDK - Advanced Microchain Example\n")

    # 1. Create a microchain with BFT consensus
    print("📦 Creating BFT microchain with custom settings...")
    config = MicrochainConfig(
        name="HighSecurityDApp",
        owner="ouro1owner...",
        consensus={"type": ConsensusType.BFT.value, "validator_count": 4},
        block_time_secs=10,
    )

    microchain = Microchain.create(config, "http://localhost:8001")
    print(f"✅ BFT Microchain ID: {microchain.id}\n")

    # 2. Build custom transaction with data payload
    print("🔨 Building custom transaction with data...")
    tx = (
        microchain.tx()
        .set_from("ouro1alice...")
        .set_to("ouro1smartcontract...")
        .set_amount(500)
        .set_data(
            {
                "method": "mint_nft",
                "params": {"token_id": "12345", "metadata": "ipfs://Qm..."},
            }
        )
        .build()
    )

    print(f"   Transaction ID: {tx.id}")
    print(f"   Data: {tx.data}\n")

    # 3. Submit custom transaction
    print("📤 Submitting custom transaction...")
    # In production: tx.sign('private_key_hex')
    tx.signature = "mock_signature"
    tx_id = microchain.submit_tx(tx)
    print(f"✅ Submitted: {tx_id}\n")

    # 4. Query transaction history
    print("📜 Fetching transaction history...")
    txs = microchain.tx_history(0, 100)
    print(f"   Found {len(txs)} transactions")
    for i, t in enumerate(txs[:5]):
        print(f"   {i + 1}. {t.from_addr} -> {t.to} ({t.amount})")
    print()

    # 5. Query recent blocks
    print("🧱 Fetching recent blocks...")
    blocks = microchain.blocks(10)
    print(f"   Retrieved {len(blocks)} blocks")
    for block in blocks:
        print(f"   Block #{block.height}: {block.tx_count} txs")
    print()

    # 6. Connect to existing microchain
    print("🔗 Connecting to existing microchain...")
    existing = Microchain.connect(microchain.id, "http://localhost:8001")
    print(f"✅ Connected to microchain: {existing.id}\n")

    print("🎉 Advanced example complete!")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"❌ Error: {e}")
        exit(1)
