// contract_templates/nft/src/lib.rs
//! OVM NFT Contract (ERC721-like)
//!
//! Non-fungible token implementation for Ouroboros Virtual Machine.
//!
//! # Features
//! - Unique token IDs
//! - Ownership tracking
//! - Approval system
//! - Metadata URIs
//! - Transfer and mint capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NFT contract state
#[derive(Serialize, Deserialize, Default)]
pub struct NFTState {
    /// Collection name (e.g., "CryptoPunks")
    pub name: String,

    /// Collection symbol (e.g., "PUNK")
    pub symbol: String,

    /// Token owner mapping: token_id -> owner_address
    pub owners: HashMap<u64, String>,

    /// Balance mapping: owner_address -> count
    pub balances: HashMap<String, u64>,

    /// Token approvals: token_id -> approved_address
    pub token_approvals: HashMap<u64, String>,

    /// Operator approvals: owner -> operator -> approved
    pub operator_approvals: HashMap<String, HashMap<String, bool>>,

    /// Token metadata URIs: token_id -> uri
    pub token_uris: HashMap<u64, String>,

    /// Contract owner (can mint)
    pub owner: String,

    /// Next token ID to mint
    pub next_token_id: u64,
}

/// Mint arguments
#[derive(Deserialize)]
pub struct MintArgs {
    pub to: String,
    pub uri: String,
}

/// Transfer arguments
#[derive(Deserialize)]
pub struct TransferArgs {
    pub to: String,
    pub token_id: u64,
}

/// Approve arguments
#[derive(Deserialize)]
pub struct ApproveArgs {
    pub to: String,
    pub token_id: u64,
}

/// SetApprovalForAll arguments
#[derive(Deserialize)]
pub struct SetApprovalForAllArgs {
    pub operator: String,
    pub approved: bool,
}

/// Initialize NFT collection
pub fn initialize(name: String, symbol: String, owner: String) -> NFTState {
    NFTState {
        name,
        symbol,
        owners: HashMap::new(),
        balances: HashMap::new(),
        token_approvals: HashMap::new(),
        operator_approvals: HashMap::new(),
        token_uris: HashMap::new(),
        owner,
        next_token_id: 1, // Start from 1
    }
}

/// Get owner of a token
pub fn owner_of(state: &NFTState, token_id: u64) -> Result<String, String> {
    state
        .owners
        .get(&token_id)
        .cloned()
        .ok_or_else(|| format!("Token {} does not exist", token_id))
}

/// Get balance of an address
pub fn balance_of(state: &NFTState, address: &str) -> u64 {
    *state.balances.get(address).unwrap_or(&0)
}

/// Get approved address for a token
pub fn get_approved(state: &NFTState, token_id: u64) -> Option<String> {
    state.token_approvals.get(&token_id).cloned()
}

/// Check if operator is approved for all tokens of owner
pub fn is_approved_for_all(state: &NFTState, owner: &str, operator: &str) -> bool {
    state
        .operator_approvals
        .get(owner)
        .and_then(|operators| operators.get(operator))
        .copied()
        .unwrap_or(false)
}

/// Get token URI
pub fn token_uri(state: &NFTState, token_id: u64) -> Result<String, String> {
    state
        .token_uris
        .get(&token_id)
        .cloned()
        .ok_or_else(|| format!("Token {} has no URI", token_id))
}

/// Mint a new NFT
pub fn mint(
    state: &mut NFTState,
    caller: &str,
    to: &str,
    uri: String,
) -> Result<u64, String> {
    if caller != state.owner {
        return Err("Only owner can mint".to_string());
    }

    let token_id = state.next_token_id;
    state.next_token_id += 1;

    // Set owner
    state.owners.insert(token_id, to.to_string());

    // Increment balance
    let balance = balance_of(state, to);
    state.balances.insert(to.to_string(), balance + 1);

    // Set URI
    state.token_uris.insert(token_id, uri.clone());

    // Emit Mint event
    println!("Mint: {} -> token #{} ({})", to, token_id, uri);

    Ok(token_id)
}

/// Transfer NFT
pub fn transfer(
    state: &mut NFTState,
    caller: &str,
    to: &str,
    token_id: u64,
) -> Result<(), String> {
    let current_owner = owner_of(state, token_id)?;

    // Check if caller is owner, approved, or operator
    let is_authorized = caller == current_owner
        || get_approved(state, token_id) == Some(caller.to_string())
        || is_approved_for_all(state, &current_owner, caller);

    if !is_authorized {
        return Err(format!(
            "Caller {} is not authorized to transfer token {}",
            caller, token_id
        ));
    }

    if to.is_empty() {
        return Err("Cannot transfer to zero address".to_string());
    }

    // Clear approval
    state.token_approvals.remove(&token_id);

    // Update balances
    let from_balance = balance_of(state, &current_owner);
    state.balances.insert(current_owner.clone(), from_balance - 1);

    let to_balance = balance_of(state, to);
    state.balances.insert(to.to_string(), to_balance + 1);

    // Update owner
    state.owners.insert(token_id, to.to_string());

    // Emit Transfer event
    println!("Transfer: {} -> {} (token #{})", current_owner, to, token_id);

    Ok(())
}

/// Approve address to transfer token
pub fn approve(
    state: &mut NFTState,
    caller: &str,
    to: &str,
    token_id: u64,
) -> Result<(), String> {
    let current_owner = owner_of(state, token_id)?;

    if caller != current_owner && !is_approved_for_all(state, &current_owner, caller) {
        return Err(format!(
            "Caller {} is not authorized to approve token {}",
            caller, token_id
        ));
    }

    state.token_approvals.insert(token_id, to.to_string());

    // Emit Approval event
    println!("Approval: token #{} -> {}", token_id, to);

    Ok(())
}

/// Set approval for all tokens
pub fn set_approval_for_all(
    state: &mut NFTState,
    caller: &str,
    operator: &str,
    approved: bool,
) -> Result<(), String> {
    if caller == operator {
        return Err("Cannot approve self as operator".to_string());
    }

    state
        .operator_approvals
        .entry(caller.to_string())
        .or_insert_with(HashMap::new)
        .insert(operator.to_string(), approved);

    // Emit ApprovalForAll event
    println!("ApprovalForAll: {} -> {} ({})", caller, operator, approved);

    Ok(())
}

/// Burn NFT (destroy permanently)
pub fn burn(state: &mut NFTState, caller: &str, token_id: u64) -> Result<(), String> {
    let current_owner = owner_of(state, token_id)?;

    if caller != current_owner {
        return Err(format!(
            "Only owner {} can burn token {}",
            current_owner, token_id
        ));
    }

    // Remove owner
    state.owners.remove(&token_id);

    // Decrease balance
    let balance = balance_of(state, caller);
    state.balances.insert(caller.to_string(), balance - 1);

    // Remove approvals
    state.token_approvals.remove(&token_id);

    // Remove URI
    state.token_uris.remove(&token_id);

    // Emit Burn event
    println!("Burn: {} burned token #{}", caller, token_id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let state = initialize(
            "Test NFT".to_string(),
            "TNFT".to_string(),
            "owner".to_string(),
        );

        assert_eq!(state.name, "Test NFT");
        assert_eq!(state.symbol, "TNFT");
        assert_eq!(state.owner, "owner");
        assert_eq!(state.next_token_id, 1);
    }

    #[test]
    fn test_mint_and_ownership() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            "owner".to_string(),
        );

        // Mint token
        let token_id = mint(&mut state, "owner", "alice", "ipfs://...".to_string()).unwrap();
        assert_eq!(token_id, 1);
        assert_eq!(owner_of(&state, token_id).unwrap(), "alice");
        assert_eq!(balance_of(&state, "alice"), 1);
        assert_eq!(token_uri(&state, token_id).unwrap(), "ipfs://...");
    }

    #[test]
    fn test_transfer() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            "owner".to_string(),
        );

        let token_id = mint(&mut state, "owner", "alice", "uri".to_string()).unwrap();

        // Alice transfers to Bob
        transfer(&mut state, "alice", "bob", token_id).unwrap();
        assert_eq!(owner_of(&state, token_id).unwrap(), "bob");
        assert_eq!(balance_of(&state, "alice"), 0);
        assert_eq!(balance_of(&state, "bob"), 1);
    }

    #[test]
    fn test_approve_and_transfer() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            "owner".to_string(),
        );

        let token_id = mint(&mut state, "owner", "alice", "uri".to_string()).unwrap();

        // Alice approves Bob
        approve(&mut state, "alice", "bob", token_id).unwrap();
        assert_eq!(get_approved(&state, token_id), Some("bob".to_string()));

        // Bob transfers to Charlie
        transfer(&mut state, "bob", "charlie", token_id).unwrap();
        assert_eq!(owner_of(&state, token_id).unwrap(), "charlie");
    }

    #[test]
    fn test_operator_approval() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            "owner".to_string(),
        );

        let token_id = mint(&mut state, "owner", "alice", "uri".to_string()).unwrap();

        // Alice sets Bob as operator
        set_approval_for_all(&mut state, "alice", "bob", true).unwrap();
        assert!(is_approved_for_all(&state, "alice", "bob"));

        // Bob can transfer Alice's token
        transfer(&mut state, "bob", "charlie", token_id).unwrap();
        assert_eq!(owner_of(&state, token_id).unwrap(), "charlie");
    }

    #[test]
    fn test_burn() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            "owner".to_string(),
        );

        let token_id = mint(&mut state, "owner", "alice", "uri".to_string()).unwrap();
        burn(&mut state, "alice", token_id).unwrap();

        assert!(owner_of(&state, token_id).is_err());
        assert_eq!(balance_of(&state, "alice"), 0);
    }

    #[test]
    fn test_unauthorized_transfer() {
        let mut state = initialize(
            "Test".to_string(),
            "TST".to_string(),
            "owner".to_string(),
        );

        let token_id = mint(&mut state, "owner", "alice", "uri".to_string()).unwrap();

        // Bob tries to transfer Alice's token
        let result = transfer(&mut state, "bob", "charlie", token_id);
        assert!(result.is_err());
    }
}
