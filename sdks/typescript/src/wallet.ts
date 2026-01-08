// sdks/typescript/src/wallet.ts
/**
 * Wallet - Key management and transaction signing
 */

import * as ed25519 from '@noble/ed25519';
import { sha256 } from '@noble/hashes/sha256';
import { Transaction, SendTransactionParams } from './transaction';

export class Wallet {
  public privateKey: Uint8Array;
  public publicKey: Uint8Array;
  public address: string;

  private constructor(privateKey: Uint8Array) {
    this.privateKey = privateKey;
    this.publicKey = ed25519.getPublicKey(privateKey);
    this.address = Buffer.from(this.publicKey).toString('hex');
  }

  /**
   * Create wallet from private key
   */
  static fromPrivateKey(privateKey: string | Uint8Array): Wallet {
    const keyBytes = typeof privateKey === 'string'
      ? Buffer.from(privateKey, 'hex')
      : privateKey;

    return new Wallet(keyBytes);
  }

  /**
   * Generate a new random wallet
   */
  static generate(): Wallet {
    const privateKey = ed25519.utils.randomPrivateKey();
    return new Wallet(privateKey);
  }

  /**
   * Sign a transaction
   */
  async signTransaction(params: SendTransactionParams): Promise<Transaction> {
    const message = `${params.from}:${params.to}:${params.amount}`;
    const messageBytes = new TextEncoder().encode(message);
    const messageHash = sha256(messageBytes);

    const signature = await ed25519.sign(messageHash, this.privateKey);

    return {
      from: params.from || this.address,
      to: params.to,
      amount: BigInt(params.amount),
      publicKey: Buffer.from(this.publicKey).toString('hex'),
      signature: Buffer.from(signature).toString('hex'),
      fee: params.fee ? BigInt(params.fee) : BigInt(1000),
    };
  }

  /**
   * Sign arbitrary data
   */
  async signData(data: Uint8Array): Promise<string> {
    const hash = sha256(data);
    const signature = await ed25519.sign(hash, this.privateKey);
    return Buffer.from(signature).toString('hex');
  }

  /**
   * Verify a signature
   */
  static async verify(
    data: Uint8Array,
    signature: string,
    publicKey: string
  ): Promise<boolean> {
    const hash = sha256(data);
    const sigBytes = Buffer.from(signature, 'hex');
    const pubKeyBytes = Buffer.from(publicKey, 'hex');

    return ed25519.verify(sigBytes, hash, pubKeyBytes);
  }
}
