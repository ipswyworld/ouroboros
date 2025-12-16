import { OuroClient, Transaction } from '../src';

/**
 * Client Usage Example
 *
 * Demonstrates using the low-level OuroClient for direct node interaction
 */
async function main() {
  console.log('🚀 Ouroboros SDK - Client Usage Example\n');

  // Create a client to interact with Ouroboros node
  const client = new OuroClient('http://localhost:8001');

  // 1. Check node health
  console.log('🏥 Checking node health...');
  const healthy = await client.healthCheck();
  console.log(`   Node status: ${healthy ? '✅ Healthy' : '❌ Unhealthy'}\n`);

  // 2. Get mainchain balance
  console.log('💰 Checking mainchain balance...');
  const balance = await client.getBalance('ouro1alice...');
  console.log(`   Address: ${balance.address}`);
  console.log(`   Balance: ${balance.balance} OURO`);
  console.log(`   Pending: ${balance.pending} OURO\n`);

  // 3. List all microchains
  console.log('📋 Listing all microchains...');
  const microchains = await client.listMicrochains();
  console.log(`   Found ${microchains.length} microchains:`);
  for (let i = 0; i < Math.min(5, microchains.length); i++) {
    const mc = microchains[i];
    console.log(`   - ${mc.id}: ${mc.name} (height: ${mc.blockHeight})`);
  }
  console.log();

  // 4. Get specific microchain state
  if (microchains.length > 0) {
    const mc = microchains[0];
    console.log('🔍 Getting microchain state...');
    const state = await client.getMicrochainState(mc.id);
    console.log(`   ID: ${state.id}`);
    console.log(`   Name: ${state.name}`);
    console.log(`   Owner: ${state.owner}`);
    console.log(`   Block Height: ${state.blockHeight}`);
    console.log(`   Total Transactions: ${state.txCount}`);
    if (state.lastAnchorHeight) {
      console.log(`   Last Anchor: Block #${state.lastAnchorHeight}`);
    }
    console.log();

    // 5. Check microchain balance
    console.log('💎 Checking microchain balance...');
    const mcBalance = await client.getMicrochainBalance(mc.id, 'ouro1alice...');
    console.log(`   Balance on ${mc.name}: ${mcBalance} tokens\n`);
  }

  // 6. Submit mainchain transaction
  console.log('📤 Submitting mainchain transaction...');
  const tx = new Transaction('ouro1alice...', 'ouro1bob...', 1000);
  // In production: tx.sign('private_key');
  tx.signature = 'mock_signature'; // For demo only

  try {
    const txId = await client.submitTransaction(tx.toJSON());
    console.log(`✅ Transaction submitted: ${txId}\n`);

    // 7. Get transaction status
    console.log('🔎 Checking transaction status...');
    const status = await client.getTransactionStatus(txId);
    console.log(`   Status: ${status}\n`);
  } catch (error) {
    console.log(`❌ Transaction failed: ${error instanceof Error ? error.message : 'Unknown'}\n`);
  }

  console.log('🎉 Client usage example complete!');
}

main().catch((error) => {
  console.error('❌ Error:', error.message);
  process.exit(1);
});
