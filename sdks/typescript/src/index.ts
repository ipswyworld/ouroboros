// sdks/typescript/src/index.ts
/**
 * Ouroboros TypeScript SDK
 *
 * JavaScript/TypeScript library for interacting with Ouroboros blockchain.
 * Similar to Web3.js for Ethereum or @solana/web3.js for Solana.
 *
 * @example
 * ```typescript
 * import { OuroborosClient, Wallet } from '@ouroboros/sdk';
 *
 * const client = new OuroborosClient('http://localhost:8001');
 * const wallet = Wallet.fromPrivateKey(privateKey);
 *
 * // Send transaction
 * const tx = await client.sendTransaction({
 *   from: wallet.address,
 *   to: recipientAddress,
 *   amount: 1000000,
 * }, wallet);
 * ```
 */

export * from './client';
export * from './wallet';
export * from './contract';
export * from './transaction';
export * from './types';
export * from './utils';
