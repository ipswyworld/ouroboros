from pydantic import BaseModel

class User(BaseModel):
    username: str

class TransactionRequest(BaseModel):
    sender: str
    recipient: str
    amount: int
