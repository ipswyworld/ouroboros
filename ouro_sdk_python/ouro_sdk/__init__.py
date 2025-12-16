"""
Ouroboros Microchain SDK for Python

Build decentralized applications on the Ouroboros blockchain platform.
"""

from .client import OuroClient
from .microchain import Microchain, MicrochainBuilder
from .transaction import Transaction, TransactionBuilder
from .types import (
    ConsensusType,
    TxStatus,
    MicrochainConfig,
    MicrochainState,
    Balance,
    BlockHeader,
    TransactionData,
    AnchorFrequency,
)
from .errors import (
    SdkError,
    NetworkError,
    TransactionFailedError,
    MicrochainNotFoundError,
    InsufficientBalanceError,
    InvalidSignatureError,
    AnchorFailedError,
    InvalidConfigError,
)

__version__ = "0.3.0"
__all__ = [
    # Core classes
    "OuroClient",
    "Microchain",
    "MicrochainBuilder",
    "Transaction",
    "TransactionBuilder",
    # Types
    "ConsensusType",
    "TxStatus",
    "MicrochainConfig",
    "MicrochainState",
    "Balance",
    "BlockHeader",
    "TransactionData",
    "AnchorFrequency",
    # Errors
    "SdkError",
    "NetworkError",
    "TransactionFailedError",
    "MicrochainNotFoundError",
    "InsufficientBalanceError",
    "InvalidSignatureError",
    "AnchorFailedError",
    "InvalidConfigError",
]
