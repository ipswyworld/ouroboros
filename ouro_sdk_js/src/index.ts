/**
 * Ouroboros Microchain SDK for JavaScript/TypeScript
 *
 * Build decentralized applications on the Ouroboros blockchain platform.
 *
 * @packageDocumentation
 */

// Core classes
export { OuroClient } from './client';
export { Microchain, MicrochainBuilder } from './microchain';
export { Transaction, TransactionBuilder } from './transaction';

// Types
export type {
  MicrochainConfig,
  MicrochainState,
  Balance,
  BlockHeader,
  TransactionData,
  AnchorFrequency,
} from './types';

export { ConsensusType, TxStatus } from './types';

// Errors
export {
  SdkError,
  NetworkError,
  TransactionFailedError,
  MicrochainNotFoundError,
  InsufficientBalanceError,
  InvalidSignatureError,
  AnchorFailedError,
  InvalidConfigError,
} from './errors';

/**
 * Default export with all SDK exports
 */
export default {
  OuroClient,
  Microchain,
  MicrochainBuilder,
  Transaction,
  TransactionBuilder,
  ConsensusType,
  TxStatus,
};
