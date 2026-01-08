# dag_auto_send.py
import requests, json, time, uuid
"""
Simple script to read dag_txn.json (array) or generate txs and POST them to /tx/submit
"""

API = "http://127.0.0.1:8080/tx/submit"  # change port if needed

def send_tx(tx_obj):
    r = requests.post(API, json=tx_obj, timeout=5)
    return r.status_code, r.text

def make_tx(sender="alice", recipient="bob", amount=1):
    return {
        "id": str(uuid.uuid4()),
        "sender": sender,
        "recipient": recipient,
        "amount": amount,
        "nonce": int(time.time()*1000)
    }

if __name__ == "__main__":
    # example: send 10 txs quickly
    for i in range(10):
        tx = make_tx(amount=i+1)
        code, text = send_tx(tx)
        print(i, code, text)
        time.sleep(0.1)
