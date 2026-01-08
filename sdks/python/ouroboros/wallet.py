"""
Ouroboros Python SDK - Wallet Module

Key management and transaction signing.
"""

import hashlib
import secrets
from typing import Optional
from dataclasses import dataclass
from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey
)
from cryptography.hazmat.primitives import serialization


@dataclass
class Transaction:
    """Signed transaction."""
    from_address: str
    to: str
    amount: int
    public_key: str
    signature: str
    fee: int


class Wallet:
    """
    Ouroboros wallet for key management and signing.

    Example:
        >>> wallet = Wallet.generate()
        >>> print(f"Address: {wallet.address}")
        >>> tx = wallet.sign_transaction(to=recipient, amount=1000000)
    """

    def __init__(self, private_key: Ed25519PrivateKey):
        """
        Initialize wallet from private key.

        Args:
            private_key: Ed25519 private key
        """
        self._private_key = private_key
        self._public_key = private_key.public_key()

        # Derive address from public key
        public_bytes = self._public_key.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
        self.address = public_bytes.hex()

    @classmethod
    def generate(cls) -> 'Wallet':
        """
        Generate a new random wallet.

        Returns:
            New wallet instance
        """
        private_key = Ed25519PrivateKey.generate()
        return cls(private_key)

    @classmethod
    def from_private_key(cls, private_key_hex: str) -> 'Wallet':
        """
        Create wallet from private key hex string.

        Args:
            private_key_hex: Private key as hex string

        Returns:
            Wallet instance
        """
        private_bytes = bytes.fromhex(private_key_hex)
        private_key = Ed25519PrivateKey.from_private_bytes(private_bytes)
        return cls(private_key)

    def sign_transaction(
        self,
        to: str,
        amount: int,
        fee: int = 1000
    ) -> Transaction:
        """
        Sign a transaction.

        Args:
            to: Recipient address
            amount: Amount in smallest units
            fee: Transaction fee

        Returns:
            Signed transaction
        """
        # Create message to sign
        message = f"{self.address}:{to}:{amount}"
        message_bytes = message.encode('utf-8')

        # Hash message
        message_hash = hashlib.sha256(message_bytes).digest()

        # Sign
        signature = self._private_key.sign(message_hash)

        return Transaction(
            from_address=self.address,
            to=to,
            amount=amount,
            public_key=self.address,  # Public key is the address
            signature=signature.hex(),
            fee=fee
        )

    def sign_data(self, data: bytes) -> str:
        """
        Sign arbitrary data.

        Args:
            data: Data to sign

        Returns:
            Signature as hex string
        """
        data_hash = hashlib.sha256(data).digest()
        signature = self._private_key.sign(data_hash)
        return signature.hex()

    @property
    def private_key_hex(self) -> str:
        """Get private key as hex string."""
        private_bytes = self._private_key.private_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PrivateFormat.Raw,
            encryption_algorithm=serialization.NoEncryption()
        )
        return private_bytes.hex()

    @staticmethod
    def verify(
        data: bytes,
        signature_hex: str,
        public_key_hex: str
    ) -> bool:
        """
        Verify a signature.

        Args:
            data: Original data
            signature_hex: Signature as hex
            public_key_hex: Public key as hex

        Returns:
            True if signature is valid
        """
        try:
            data_hash = hashlib.sha256(data).digest()
            signature = bytes.fromhex(signature_hex)
            public_bytes = bytes.fromhex(public_key_hex)

            public_key = Ed25519PublicKey.from_public_bytes(public_bytes)
            public_key.verify(signature, data_hash)
            return True
        except Exception:
            return False
