"""
Client Usage Example

Demonstrates using the low-level OuroClient for direct node interaction
"""

from ouro_sdk import OuroClient, Transaction


def main():
    print("🚀 Ouroboros SDK - Client Usage Example\n")

    # Create a client to interact with Ouroboros node
    client = OuroClient("http://localhost:8001")

    # 1. Check node health
    print("🏥 Checking node health...")
    healthy = client.health_check()
    print(f"   Node status: {'✅ Healthy' if healthy else '❌ Unhealthy'}\n")

    # 2. Get mainchain balance
    print("💰 Checking mainchain balance...")
    balance = client.get_balance("ouro1alice...")
    print(f"   Address: {balance.address}")
    print(f"   Balance: {balance.balance} OURO")
    print(f"   Pending: {balance.pending} OURO\n")

    # 3. List all microchains
    print("📋 Listing all microchains...")
    microchains = client.list_microchains()
    print(f"   Found {len(microchains)} microchains:")
    for mc in microchains[:5]:
        print(f"   - {mc.id}: {mc.name} (height: {mc.block_height})")
    print()

    # 4. Get specific microchain state
    if microchains:
        mc = microchains[0]
        print("🔍 Getting microchain state...")
        state = client.get_microchain_state(mc.id)
        print(f"   ID: {state.id}")
        print(f"   Name: {state.name}")
        print(f"   Owner: {state.owner}")
        print(f"   Block Height: {state.block_height}")
        print(f"   Total Transactions: {state.tx_count}")
        if state.last_anchor_height:
            print(f"   Last Anchor: Block #{state.last_anchor_height}")
        print()

        # 5. Check microchain balance
        print("💎 Checking microchain balance...")
        mc_balance = client.get_microchain_balance(mc.id, "ouro1alice...")
        print(f"   Balance on {mc.name}: {mc_balance} tokens\n")

    # 6. Submit mainchain transaction
    print("📤 Submitting mainchain transaction...")
    tx = Transaction("ouro1alice...", "ouro1bob...", 1000)
    # In production: tx.sign('private_key')
    tx.signature = "mock_signature"  # For demo only

    try:
        tx_id = client.submit_transaction(tx.to_json())
        print(f"✅ Transaction submitted: {tx_id}\n")

        # 7. Get transaction status
        print("🔎 Checking transaction status...")
        status = client.get_transaction_status(tx_id)
        print(f"   Status: {status.value}\n")
    except Exception as e:
        print(f"❌ Transaction failed: {e}\n")

    print("🎉 Client usage example complete!")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"❌ Error: {e}")
        exit(1)
