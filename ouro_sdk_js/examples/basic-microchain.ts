import { Microchain, MicrochainBuilder, ConsensusType } from '../src';

/**
 * Basic Microchain Example
 *
 * Demonstrates:
 * - Creating a microchain
 * - Checking state and balances
 * - Sending transactions
 * - Anchoring to mainchain
 */
async function main() {
  console.log('🚀 Ouroboros SDK - Basic Microchain Example\n');

  // 1. Create a new microchain with default settings
  console.log('📦 Creating new microchain...');
  const config = {
    name: 'MyDApp',
    owner: 'ouro1owner123...',
    blockTimeSecs: 5, // 5 second blocks
  };

  const microchain = await Microchain.create(config, 'http://localhost:8001');
  console.log(`✅ Microchain created with ID: ${microchain.id}\n`);

  // 2. Check microchain state
  console.log('🔍 Fetching microchain state...');
  const state = await microchain.state();
  console.log(`   Name: ${state.name}`);
  console.log(`   Owner: ${state.owner}`);
  console.log(`   Block Height: ${state.blockHeight}`);
  console.log(`   Total Transactions: ${state.txCount}\n`);

  // 3. Check balance
  console.log('💰 Checking balance...');
  const balance = await microchain.balance('ouro1owner123...');
  console.log(`   Balance: ${balance} OURO\n`);

  // 4. Transfer tokens
  console.log('💸 Sending transaction...');
  const txId = await microchain.transfer(
    'ouro1owner123...',
    'ouro1recipient456...',
    1000
  );
  console.log(`✅ Transaction submitted: ${txId}\n`);

  // 5. Anchor to mainchain for security
  console.log('⚓ Anchoring to mainchain...');
  const anchorId = await microchain.anchor();
  console.log(`✅ Anchored with ID: ${anchorId}\n`);

  console.log('🎉 Example complete!');
}

main().catch((error) => {
  console.error('❌ Error:', error.message);
  process.exit(1);
});
