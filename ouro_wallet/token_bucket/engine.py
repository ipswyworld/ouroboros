import json
import os
from datetime import datetime, timedelta

# In-memory store for simplicity (replace with DB later)
token_balances = {}     # { "alice": { "balance": 100, "last_issued": "2025-06-30" } }
token_spends = []       # list of { sender, recipient, amount, timestamp }

DAILY_QUOTA = 100

def issue_tokens(username: str):
    today = datetime.utcnow().date().isoformat()
    user = token_balances.get(username)

    if not user or user["last_issued"] != today:
        token_balances[username] = {
            "balance": DAILY_QUOTA,
            "last_issued": today
        }

def get_token_balance(username: str):
    issue_tokens(username)
    return token_balances[username]["balance"]

def spend_token(sender: str, recipient: str, amount: int):
    issue_tokens(sender)
    if token_balances[sender]["balance"] < amount:
        return False

    token_balances[sender]["balance"] -= amount
    spend_record = {
        "sender": sender,
        "recipient": recipient,
        "amount": amount,
        "timestamp": datetime.utcnow().isoformat()
    }
    token_spends.append(spend_record)

    with open("token_spends.json", "w") as f:
        json.dump(token_spends, f, indent=2)

    return True
