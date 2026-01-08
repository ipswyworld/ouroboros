from fastapi import FastAPI, HTTPException
from models import User, TransactionRequest
from services import create_user, send_tokens, get_balance

app = FastAPI()

@app.post("/register")
def register(user: User):
    create_user(user.username)
    return {"message": "User registered!"}

@app.get("/balance/{username}")
def balance(username: str):
    bal = get_balance(username)
    return {"username": username, "balance": bal}

@app.post("/send")
def send(request: TransactionRequest):
    result = send_tokens(request)
    if not result:
        raise HTTPException(status_code=400, detail="Transaction failed")
    return {"message": "Transaction submitted!"}

@app.get("/history")
def history():
    return {"transactions": get_history()}

from token_bucket.engine import spend_token, get_token_balance

@app.post("/spend_token")
def spend(request: TransactionRequest):
    success = spend_token(request.sender, request.recipient, request.amount)
    if not success:
        raise HTTPException(status_code=400, detail="Insufficient tokens")
    return {"message": "Token spend recorded."}

@app.get("/token_balance/{username}")
def token_balance(username: str):
    bal = get_token_balance(username)
    return {"username": username, "token_balance": bal}
