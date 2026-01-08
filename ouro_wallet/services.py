import json
from storage import balances, transactions
from models import TransactionRequest

def create_user(username: str):
    if username not in balances:
        balances[username] = 0

def get_balance(username: str) -> int:
    return balances.get(username, 0)

def send_tokens(request: TransactionRequest) -> bool:
    if balances.get(request.sender, 0) < request.amount:
        return False

    balances[request.sender] -= request.amount
    balances[request.recipient] = balances.get(request.recipient, 0) + request.amount

    # Record txn in memory
    transactions.append({
        "from": request.sender,
        "to": request.recipient,
        "amount": request.amount
    })

    # Write to file so Rust can consume
    txn = {
        "sender": request.sender,
        "recipient": request.recipient,
        "amount": request.amount
    }

    with open("dag_txn.json", "w") as f:
        json.dump(txn, f)

    return True

from dag_bridge.dag_simulator import get_dag_balances, get_dag_history

def get_balance(username: str) -> int:
    return get_dag_balances().get(username, 0)

def get_history():
    return get_dag_history()

from crypto_utils import generate_keypair, sign_message

# In-memory key store
user_keys = {}

def create_user(username: str):
    if username not in balances:
        balances[username] = 0
        user_keys[username] = generate_keypair()

def get_pubkey(username: str):
    return user_keys.get(username, {}).get("public_key", "no-key")

def send_tokens(request: TransactionRequest) -> bool:
    if balances.get(request.sender, 0) < request.amount:
        return False

    balances[request.sender] -= request.amount
    balances[request.recipient] = balances.get(request.recipient, 0) + request.amount

    # Create message and sign
    msg = f"{request.sender}:{request.recipient}:{request.amount}"
    priv = user_keys[request.sender]["private_key"]
    pub = user_keys[request.sender]["public_key"]
    sig = sign_message(msg, priv)

    txn = {
        "sender": request.sender,
        "recipient": request.recipient,
        "amount": request.amount,
        "public_key": pub,
        "signature": sig
    }

    with open("dag_txn.json", "w") as f:
        json.dump(txn, f)

    return True
