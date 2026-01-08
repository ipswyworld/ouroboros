// contract_templates/token/src/lib.rs
//! OVM Token Contract (ERC20-like)
//!
//! A fungible token implementation for Ouroboros Virtual Machine.
//!
//! # Features
//! - Standard ERC20-like interface (transfer, approve, transferFrom)
//! - Minting and burning capabilities
//! - Event emission for indexing
//! - Gas-optimized storage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token contract state
#[derive(Serialize, Deserialize, Default)]
pub struct TokenState {
    /// Token name (e.g., "My Token")
    pub name: String,

    /// Token symbol (e.g., "MTK")
    pub symbol: String,

    /// Number of decimals (e.g., 18)
    pub decimals: u8,

    /// Total supply
    pub total_supply: u64,

    /// Balance mapping: address -> amount
    pub balances: HashMap<String, u64>,

    /// Allowance mapping: owner -> spender -> amount
    pub allowances: HashMap<String, HashMap<String, u64>>,

    /// Contract owner (can mint/burn)
    pub owner: String,
}

/// Transfer arguments
#[derive(Deserialize)]
pub struct TransferArgs {
    pub to: String,
    pub amount: u64,
}

/// Approve arguments
#[derive(Deserialize)]
pub struct ApproveArgs {
    pub spender: String,
    pub amount: u64,
}

/// TransferFrom arguments
#[derive(Deserialize)]
pub struct TransferFromArgs {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

/// Mint arguments (owner only)
#[derive(Deserialize)]
pub struct MintArgs {
    pub to: String,
    pub amount: u64,
}

/// Burn arguments
#[derive(Deserialize)]
pub struct BurnArgs {
    pub amount: u64,
}

/// Contract initialization
pub fn initialize(name: String, symbol: String, decimals: u8, owner: String) -> TokenState {
    TokenState {
        name,
        symbol,
        decimals,
        total_supply: 0,
        balances: HashMap::new(),
        allowances: HashMap::new(),
        owner,
    }
}

/// Get balance of an address
pub fn balance_of(state: &TokenState, address: &str) -> u64 {
    *state.balances.get(address).unwrap_or(&0)
}

/// Get allowance for spender
pub fn allowance(state: &TokenState, owner: &str, spender: &str) -> u64 {
    state
        .allowances
        .get(owner)
        .and_then(|spenders| spenders.get(spender))
        .copied()
        .unwrap_or(0)
}

/// Transfer tokens
pub fn transfer(
    state: &mut TokenState,
    caller: &str,
    to: &str,
    amount: u64,
) -> Result<(), String> {
    if caller == to {
        return Err("Cannot transfer to self".to_string());
    }

    let sender_balance = balance_of(state, caller);
    if sender_balance < amount {
        return Err(format!(
            "Insufficient balance: {} < {}",
            sender_balance, amount
        ));
    }

    // Deduct from sender
    state.balances.insert(caller.to_string(), sender_balance - amount);

    // Add to recipient
    let recipient_balance = balance_of(state, to);
    state.balances.insert(to.to_string(), recipient_balance + amount);

    // Emit Transfer event
    println!("Transfer: {} -> {} ({})", caller, to, amount);

    Ok(())
}

/// Approve spender
pub fn approve(
    state: &mut TokenState,
    caller: &str,
    spender: &str,
    amount: u64,
) -> Result<(), String> {
    state
        .allowances
        .entry(caller.to_string())
        .or_insert_with(HashMap::new)
        .insert(spender.to_string(), amount);

    // Emit Approval event
    println!("Approval: {} -> {} ({})", caller, spender, amount);

    Ok(())
}

/// Transfer from (using allowance)
pub fn transfer_from(
    state: &mut TokenState,
    caller: &str,
    from: &str,
    to: &str,
    amount: u64,
) -> Result<(), String> {
    // Check allowance
    let allowed = allowance(state, from, caller);
    if allowed < amount {
        return Err(format!(
            "Allowance exceeded: {} < {}",
            allowed, amount
        ));
    }

    // Check balance
    let from_balance = balance_of(state, from);
    if from_balance < amount {
        return Err(format!(
            "Insufficient balance: {} < {}",
            from_balance, amount
        ));
    }

    // Deduct allowance
    state
        .allowances
        .get_mut(from)
        .unwrap()
        .insert(caller.to_string(), allowed - amount);

    // Deduct from sender
    state.balances.insert(from.to_string(), from_balance - amount);

    // Add to recipient
    let to_balance = balance_of(state, to);
    state.balances.insert(to.to_string(), to_balance + amount);

    // Emit Transfer event
    println!("Transfer: {} -> {} ({}) via {}", from, to, amount, caller);

    Ok(())
}

/// Mint tokens (owner only)
pub fn mint(
    state: &mut TokenState,
    caller: &str,
    to: &str,
    amount: u64,
) -> Result<(), String> {
    if caller != state.owner {
        return Err("Only owner can mint".to_string());
    }

    let to_balance = balance_of(state, to);
    state.balances.insert(to.to_string(), to_balance + amount);
    state.total_supply += amount;

    // Emit Mint event
    println!("Mint: {} ({}) total: {}", to, amount, state.total_supply);

    Ok(())
}

/// Burn tokens
pub fn burn(state: &mut TokenState, caller: &str, amount: u64) -> Result<(), String> {
    let caller_balance = balance_of(state, caller);
    if caller_balance < amount {
        return Err(format!(
            "Insufficient balance to burn: {} < {}",
            caller_balance, amount
        ));
    }

    state.balances.insert(caller.to_string(), caller_balance - amount);
    state.total_supply -= amount;

    // Emit Burn event
    println!("Burn: {} ({}) total: {}", caller, amount, state.total_supply);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let state = initialize(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            "owner".to_string(),
        );

        assert_eq!(state.name, "Test Token");
        assert_eq!(state.symbol, "TEST");
        assert_eq!(state.decimals, 18);
        assert_eq!(state.total_supply, 0);
        assert_eq!(state.owner, "owner");
    }

    #[test]
    fn test_mint_and_transfer() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            18,
            "owner".to_string(),
        );

        // Mint tokens
        mint(&mut state, "owner", "alice", 1000).unwrap();
        assert_eq!(balance_of(&state, "alice"), 1000);
        assert_eq!(state.total_supply, 1000);

        // Transfer
        transfer(&mut state, "alice", "bob", 300).unwrap();
        assert_eq!(balance_of(&state, "alice"), 700);
        assert_eq!(balance_of(&state, "bob"), 300);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            18,
            "owner".to_string(),
        );

        // Mint to alice
        mint(&mut state, "owner", "alice", 1000).unwrap();

        // Alice approves bob to spend 500
        approve(&mut state, "alice", "bob", 500).unwrap();
        assert_eq!(allowance(&state, "alice", "bob"), 500);

        // Bob transfers from alice to charlie
        transfer_from(&mut state, "bob", "alice", "charlie", 300).unwrap();
        assert_eq!(balance_of(&state, "alice"), 700);
        assert_eq!(balance_of(&state, "charlie"), 300);
        assert_eq!(allowance(&state, "alice", "bob"), 200);
    }

    #[test]
    fn test_burn() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            18,
            "owner".to_string(),
        );

        mint(&mut state, "owner", "alice", 1000).unwrap();
        burn(&mut state, "alice", 300).unwrap();

        assert_eq!(balance_of(&state, "alice"), 700);
        assert_eq!(state.total_supply, 700);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            18,
            "owner".to_string(),
        );

        mint(&mut state, "owner", "alice", 100).unwrap();

        let result = transfer(&mut state, "alice", "bob", 200);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient balance"));
    }

    #[test]
    fn test_only_owner_can_mint() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            18,
            "owner".to_string(),
        );

        let result = mint(&mut state, "alice", "bob", 1000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Only owner can mint"));
    }
}
