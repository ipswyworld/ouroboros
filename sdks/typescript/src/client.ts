// sdks/typescript/src/client.ts
/**
 * Ouroboros Client - Main entry point for blockchain interactions
 */

import axios, { AxiosInstance } from 'axios';
import { Transaction, TransactionReceipt, SendTransactionParams } from './transaction';
import { Wallet } from './wallet';
import { Contract, ContractABI } from './contract';

export interface ClientConfig {
  /** RPC endpoint URL */
  url: string;

  /** API key for authentication (optional) */
  apiKey?: string;

  /** Request timeout in milliseconds */
  timeout?: number;
}

export class OuroborosClient {
  private axios: AxiosInstance;
  public url: string;

  constructor(config: string | ClientConfig) {
    if (typeof config === 'string') {
      this.url = config;
      this.axios = axios.create({
        baseURL: config,
        timeout: 30000,
      });
    } else {
      this.url = config.url;
      this.axios = axios.create({
        baseURL: config.url,
        timeout: config.timeout || 30000,
        headers: config.apiKey
          ? { 'Authorization': `Bearer ${config.apiKey}` }
          : {},
      });
    }
  }

  /**
   * Get current block height
   */
  async getBlockHeight(): Promise<number> {
    const response = await this.axios.get('/status');
    return response.data.blockHeight || 0;
  }

  /**
   * Get balance for an address
   */
  async getBalance(address: string): Promise<bigint> {
    const response = await this.axios.get(`/ouro/balance/${address}`);
    return BigInt(response.data.balance || 0);
  }

  /**
   * Send a transaction
   */
  async sendTransaction(
    params: SendTransactionParams,
    wallet: Wallet
  ): Promise<TransactionReceipt> {
    const tx = await wallet.signTransaction(params);

    const response = await this.axios.post('/submit_txn', {
      sender: tx.from,
      recipient: tx.to,
      amount: tx.amount.toString(),
      public_key: tx.publicKey,
      signature: tx.signature,
      fee: tx.fee?.toString() || '1000',
    });

    return {
      txHash: response.data.tx_id || response.data.txid,
      status: 'pending',
      blockHeight: await this.getBlockHeight(),
    };
  }

  /**
   * Deploy a smart contract
   */
  async deployContract(
    code: Uint8Array,
    wallet: Wallet,
    name?: string,
    version?: string
  ): Promise<string> {
    const response = await this.axios.post('/vm/deploy', {
      code: Buffer.from(code).toString('hex'),
      deployer: wallet.address,
      name,
      version,
    });

    return response.data.address;
  }

  /**
   * Call a contract method
   */
  async callContract(
    address: string,
    method: string,
    args: any[],
    wallet?: Wallet
  ): Promise<any> {
    const response = await this.axios.post('/vm/call', {
      contract_address: address,
      method,
      args: JSON.stringify(args),
      caller: wallet?.address,
    });

    return response.data.result;
  }

  /**
   * Create a contract instance
   */
  contract(address: string, abi: ContractABI): Contract {
    return new Contract(this, address, abi);
  }

  /**
   * Query subchain info
   */
  async getSubchainInfo(subchainId: string): Promise<any> {
    const response = await this.axios.get(`/subchain/${subchainId}`);
    return response.data;
  }

  /**
   * Query microchain info
   */
  async getMicrochainInfo(microchainId: string): Promise<any> {
    const response = await this.axios.get(`/microchain/${microchainId}`);
    return response.data;
  }

  /**
   * Get data availability info
   */
  async getBlockData(blockHash: string, cid?: string): Promise<any> {
    const params = cid ? `?cid=${cid}` : '';
    const response = await this.axios.get(`/da/data/${blockHash}${params}`);
    return response.data;
  }
}
