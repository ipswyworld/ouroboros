import { Microchain, ConsensusType, Transaction } from '../src';

/**
 * Advanced Microchain Example
 *
 * Demonstrates:
 * - BFT consensus configuration
 * - Custom transactions with data payloads
 * - Transaction history queries
 * - Block queries
 * - Connecting to existing microchains
 */
async function main() {
  console.log('🚀 Ouroboros SDK - Advanced Microchain Example\n');

  // 1. Create a microchain with BFT consensus
  console.log('📦 Creating BFT microchain with custom settings...');
  const config = {
    name: 'HighSecurityDApp',
    owner: 'ouro1owner...',
    consensus: {
      type: ConsensusType.Bft,
      validatorCount: 4,
    },
    anchorFrequency: { type: 'blocks' as const, count: 100 },
    blockTimeSecs: 10,
  };

  const microchain = await Microchain.create(config, 'http://localhost:8001');
  console.log(`✅ BFT Microchain ID: ${microchain.id}\n`);

  // 2. Build custom transaction with data payload
  console.log('🔨 Building custom transaction with data...');
  const tx = microchain
    .tx()
    .setFrom('ouro1alice...')
    .setTo('ouro1smartcontract...')
    .setAmount(500)
    .setData({
      method: 'mint_nft',
      params: {
        token_id: '12345',
        metadata: 'ipfs://Qm...',
      },
    })
    .build();

  console.log(`   Transaction ID: ${tx.id}`);
  console.log(`   Data:`, tx.data, '\n');

  // 3. Submit custom transaction
  console.log('📤 Submitting custom transaction...');
  // In production: tx.sign('private_key_hex');
  tx.signature = 'mock_signature';
  const txId = await microchain.submitTx(tx);
  console.log(`✅ Submitted: ${txId}\n`);

  // 4. Query transaction history
  console.log('📜 Fetching transaction history...');
  const txs = await microchain.txHistory(0, 100);
  console.log(`   Found ${txs.length} transactions`);
  for (let i = 0; i < Math.min(5, txs.length); i++) {
    const t = txs[i];
    console.log(`   ${i + 1}. ${t.from} -> ${t.to} (${t.amount})`);
  }
  console.log();

  // 5. Query recent blocks
  console.log('🧱 Fetching recent blocks...');
  const blocks = await microchain.blocks(10);
  console.log(`   Retrieved ${blocks.length} blocks`);
  for (const block of blocks) {
    console.log(`   Block #${block.height}: ${block.txCount} txs`);
  }
  console.log();

  // 6. Connect to existing microchain
  console.log('🔗 Connecting to existing microchain...');
  const existing = await Microchain.connect(microchain.id, 'http://localhost:8001');
  console.log(`✅ Connected to microchain: ${existing.id}\n`);

  console.log('🎉 Advanced example complete!');
}

main().catch((error) => {
  console.error('❌ Error:', error.message);
  process.exit(1);
});
