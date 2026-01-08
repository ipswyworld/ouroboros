import json
import os

def get_dag_balances():
    if not os.path.exists("dag_state.json"):
        return {}

    with open("dag_state.json", "r") as f:
        state = json.load(f)
    return state.get("balances", {})

def get_dag_history():
    if not os.path.exists("dag_state.json"):
        return []

    with open("dag_state.json", "r") as f:
        state = json.load(f)
    return state.get("transactions", [])
