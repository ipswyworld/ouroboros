import json
from storage import balances, transactions
from models import TransactionRequest
from crypto_utils import generate_keypair, sign_message

# In-memory key store
user_keys = {}

def create_user(username: str):
    """Create a new user with balance and keypair"""
    if username not in balances:
        balances[username] = 0
        user_keys[username] = generate_keypair()

def get_balance(username: str) -> int:
    """Get user balance"""
    return balances.get(username, 0)

def get_pubkey(username: str):
    """Get user's public key"""
    return user_keys.get(username, {}).get("public_key", "no-key")

def get_history():
    """Get transaction history"""
    return transactions

def send_tokens(request: TransactionRequest) -> bool:
    """Send tokens from one user to another with signature"""
    # Check sufficient balance
    if balances.get(request.sender, 0) < request.amount:
        return False

    # Check sender has keys
    if request.sender not in user_keys:
        return False

    # Update balances
    balances[request.sender] -= request.amount
    balances[request.recipient] = balances.get(request.recipient, 0) + request.amount

    # Create and sign transaction
    msg = f"{request.sender}:{request.recipient}:{request.amount}"
    priv = user_keys[request.sender]["private_key"]
    pub = user_keys[request.sender]["public_key"]
    sig = sign_message(msg, priv)

    # Create transaction object
    txn = {
        "sender": request.sender,
        "recipient": request.recipient,
        "amount": request.amount,
        "public_key": pub,
        "signature": sig
    }

    # Record in memory
    transactions.append(txn)

    # Write to file for Rust DAG to consume
    with open("dag_txn.json", "w") as f:
        json.dump(txn, f)

    return True
