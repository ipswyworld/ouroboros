from nacl.signing import SigningKey
from nacl.encoding import HexEncoder

def generate_keypair():
    sk = SigningKey.generate()
    vk = sk.verify_key
    return {
        "private_key": sk.encode(encoder=HexEncoder).decode(),
        "public_key": vk.encode(encoder=HexEncoder).decode()
    }

def sign_message(message: str, private_key: str) -> str:
    sk = SigningKey(private_key, encoder=HexEncoder)
    signed = sk.sign(message.encode())
    return signed.signature.hex()
