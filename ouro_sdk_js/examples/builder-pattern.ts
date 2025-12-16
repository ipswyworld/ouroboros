import { MicrochainBuilder, ConsensusType } from '../src';

/**
 * Builder Pattern Example
 *
 * Demonstrates the fluent API for creating microchains with different configurations
 */
async function main() {
  console.log('🚀 Ouroboros SDK - Builder Pattern Example\n');

  // Example 1: Simple microchain
  console.log('📦 Creating simple microchain with builder...');
  const simple = await new MicrochainBuilder('SimpleDApp', 'ouro1owner...')
    .node('http://localhost:8001')
    .build();

  console.log(`✅ Created: ${simple.id}\n`);

  // Example 2: Gaming microchain with fast blocks
  console.log('🎮 Creating gaming microchain...');
  const gaming = await new MicrochainBuilder('GameFi', 'ouro1gamedev...')
    .node('http://localhost:8001')
    .blockTime(2) // 2 second blocks for responsive gameplay
    .consensus(ConsensusType.SingleValidator) // Fast, centralized
    .anchorFrequency({ type: 'seconds', count: 300 }) // Anchor every 5 minutes
    .build();

  console.log(`✅ Gaming microchain: ${gaming.id}`);
  console.log('   Block time: 2 seconds');
  console.log('   Consensus: SingleValidator (fast)\n');

  // Example 3: DeFi microchain with high security
  console.log('💰 Creating DeFi microchain...');
  const defi = await new MicrochainBuilder('DeFiProtocol', 'ouro1defi...')
    .node('http://localhost:8001')
    .blockTime(10) // 10 second blocks for stability
    .consensus(ConsensusType.Bft, 7) // High security with 7 validators
    .anchorFrequency({ type: 'blocks', count: 50 }) // Frequent anchoring
    .build();

  console.log(`✅ DeFi microchain: ${defi.id}`);
  console.log('   Block time: 10 seconds');
  console.log('   Consensus: BFT with 7 validators');
  console.log('   Anchoring: Every 50 blocks\n');

  // Example 4: Manual anchoring microchain
  console.log('🔧 Creating microchain with manual anchoring...');
  const manual = await new MicrochainBuilder('ManualChain', 'ouro1manual...')
    .node('http://localhost:8001')
    .anchorFrequency({ type: 'manual' })
    .build();

  console.log(`✅ Manual microchain: ${manual.id}`);
  console.log('   Anchoring: Manual only\n');

  console.log('🎉 Builder pattern examples complete!');
}

main().catch((error) => {
  console.error('❌ Error:', error.message);
  process.exit(1);
});
