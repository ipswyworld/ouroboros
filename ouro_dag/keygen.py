import nacl.signing, binascii
sk = nacl.signing.SigningKey.generate()
pk = sk.verify_key
secret = sk.encode()  # 32 bytes
pub = pk.encode()     # 32 bytes
pair = secret + pub
print(pair.hex())
