"""
Ouroboros Python SDK - Client Module

Main entry point for blockchain interactions.
Similar to web3.py for Ethereum.
"""

import requests
from typing import Optional, Dict, Any, List
from .wallet import Wallet
from .transaction import Transaction, TransactionReceipt
from .contract import Contract, ContractABI


class OuroborosClient:
    """
    Ouroboros blockchain client.

    Example:
        >>> client = OuroborosClient('http://localhost:8001')
        >>> balance = await client.get_balance(address)
        >>> print(f"Balance: {balance} OURO")
    """

    def __init__(
        self,
        url: str,
        api_key: Optional[str] = None,
        timeout: int = 30
    ):
        """
        Initialize client.

        Args:
            url: RPC endpoint URL
            api_key: Optional API key for authentication
            timeout: Request timeout in seconds
        """
        self.url = url.rstrip('/')
        self.timeout = timeout
        self.session = requests.Session()

        if api_key:
            self.session.headers['Authorization'] = f'Bearer {api_key}'

    def get_block_height(self) -> int:
        """Get current block height."""
        response = self.session.get(
            f'{self.url}/status',
            timeout=self.timeout
        )
        response.raise_for_status()
        return response.json().get('blockHeight', 0)

    def get_balance(self, address: str) -> int:
        """
        Get balance for an address.

        Args:
            address: Account address

        Returns:
            Balance in smallest units
        """
        response = self.session.get(
            f'{self.url}/ouro/balance/{address}',
            timeout=self.timeout
        )
        response.raise_for_status()
        return int(response.json().get('balance', 0))

    def send_transaction(
        self,
        to: str,
        amount: int,
        wallet: Wallet,
        fee: int = 1000
    ) -> TransactionReceipt:
        """
        Send a transaction.

        Args:
            to: Recipient address
            amount: Amount in smallest units
            wallet: Sender wallet
            fee: Transaction fee

        Returns:
            Transaction receipt
        """
        tx = wallet.sign_transaction(
            to=to,
            amount=amount,
            fee=fee
        )

        response = self.session.post(
            f'{self.url}/submit_txn',
            json={
                'sender': tx.from_address,
                'recipient': tx.to,
                'amount': str(tx.amount),
                'public_key': tx.public_key,
                'signature': tx.signature,
                'fee': str(tx.fee),
            },
            timeout=self.timeout
        )
        response.raise_for_status()

        data = response.json()
        return TransactionReceipt(
            tx_hash=data.get('tx_id') or data.get('txid'),
            status='pending',
            block_height=self.get_block_height()
        )

    def deploy_contract(
        self,
        code: bytes,
        wallet: Wallet,
        name: Optional[str] = None,
        version: Optional[str] = None
    ) -> str:
        """
        Deploy a smart contract.

        Args:
            code: WASM bytecode
            wallet: Deployer wallet
            name: Contract name
            version: Contract version

        Returns:
            Contract address
        """
        response = self.session.post(
            f'{self.url}/vm/deploy',
            json={
                'code': code.hex(),
                'deployer': wallet.address,
                'name': name,
                'version': version,
            },
            timeout=self.timeout
        )
        response.raise_for_status()

        return response.json()['address']

    def call_contract(
        self,
        address: str,
        method: str,
        args: List[Any],
        wallet: Optional[Wallet] = None
    ) -> Any:
        """
        Call a contract method.

        Args:
            address: Contract address
            method: Method name
            args: Method arguments
            wallet: Optional caller wallet

        Returns:
            Method return value
        """
        import json

        response = self.session.post(
            f'{self.url}/vm/call',
            json={
                'contract_address': address,
                'method': method,
                'args': json.dumps(args),
                'caller': wallet.address if wallet else None,
            },
            timeout=self.timeout
        )
        response.raise_for_status()

        return response.json()['result']

    def contract(self, address: str, abi: ContractABI) -> Contract:
        """
        Create a contract instance.

        Args:
            address: Contract address
            abi: Contract ABI

        Returns:
            Contract instance
        """
        return Contract(self, address, abi)

    def get_subchain_info(self, subchain_id: str) -> Dict[str, Any]:
        """Get subchain information."""
        response = self.session.get(
            f'{self.url}/subchain/{subchain_id}',
            timeout=self.timeout
        )
        response.raise_for_status()
        return response.json()

    def get_microchain_info(self, microchain_id: str) -> Dict[str, Any]:
        """Get microchain information."""
        response = self.session.get(
            f'{self.url}/microchain/{microchain_id}',
            timeout=self.timeout
        )
        response.raise_for_status()
        return response.json()

    def get_block_data(
        self,
        block_hash: str,
        cid: Optional[str] = None
    ) -> Dict[str, Any]:
        """
        Get block data from data availability layer.

        Args:
            block_hash: Block hash
            cid: Optional IPFS CID

        Returns:
            Block data
        """
        url = f'{self.url}/da/data/{block_hash}'
        if cid:
            url += f'?cid={cid}'

        response = self.session.get(url, timeout=self.timeout)
        response.raise_for_status()
        return response.json()
