#!/usr/bin/env python3
# sign_and_submit.py
import nacl.signing, nacl.encoding, requests, json, sys

API = "http://127.0.0.1:8001/tx/submit"

# Example fields
sender = "alice"
recipient = "bob"
amount = 7

# create deterministic keypair for test (DO NOT USE FOR PROD)
seed = bytes([1]*32)
sk = nacl.signing.SigningKey(seed)
pk = sk.verify_key

message = f"{sender}:{recipient}:{amount}".encode()

sig = sk.sign(message).signature
pub_hex = pk.encode(encoder=nacl.encoding.HexEncoder).decode()
sig_hex = nacl.encoding.HexEncoder.encode(sig).decode()

payload = {
    "sender": sender,
    "recipient": recipient,
    "amount": amount,
    "public_key": pub_hex,
    "signature": sig_hex
}

print("Submitting payload:", json.dumps(payload))
res = requests.post(API, json=payload, timeout=10)
print("Status:", res.status_code)
try:
    print("Response:", res.json())
except:
    print("Response text:", res.text)
