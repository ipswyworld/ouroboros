// contract_templates/simple_dex/src/lib.rs
//! Simple DEX Contract for OVM
//!
//! A basic automated market maker (AMM) decentralized exchange.
//!
//! # Features
//! - Token pair liquidity pools
//! - Constant product formula (x * y = k)
//! - Add/remove liquidity
//! - Token swapping
//! - Liquidity provider shares

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DEX contract state
#[derive(Serialize, Deserialize, Default)]
pub struct DEXState {
    /// Liquidity pools: (token_a, token_b) -> Pool
    pub pools: HashMap<(String, String), LiquidityPool>,

    /// LP token balances: user -> (token_a, token_b) -> shares
    pub lp_balances: HashMap<String, HashMap<(String, String), u64>>,

    /// Owner address
    pub owner: String,

    /// Fee basis points (e.g., 30 = 0.3%)
    pub fee_basis_points: u64,
}

/// Liquidity pool for a token pair
#[derive(Serialize, Deserialize, Clone)]
pub struct LiquidityPool {
    /// Reserve of token A
    pub reserve_a: u64,

    /// Reserve of token B
    pub reserve_b: u64,

    /// Total LP shares
    pub total_shares: u64,
}

impl LiquidityPool {
    fn new() -> Self {
        Self {
            reserve_a: 0,
            reserve_b: 0,
            total_shares: 0,
        }
    }
}

/// Add liquidity arguments
#[derive(Deserialize)]
pub struct AddLiquidityArgs {
    pub token_a: String,
    pub token_b: String,
    pub amount_a: u64,
    pub amount_b: u64,
}

/// Remove liquidity arguments
#[derive(Deserialize)]
pub struct RemoveLiquidityArgs {
    pub token_a: String,
    pub token_b: String,
    pub shares: u64,
}

/// Swap arguments
#[derive(Deserialize)]
pub struct SwapArgs {
    pub token_in: String,
    pub token_out: String,
    pub amount_in: u64,
    pub min_amount_out: u64, // Slippage protection
}

/// Initialize DEX
pub fn initialize(owner: String, fee_basis_points: u64) -> DEXState {
    DEXState {
        pools: HashMap::new(),
        lp_balances: HashMap::new(),
        owner,
        fee_basis_points, // e.g., 30 = 0.3%
    }
}

/// Get pool for token pair (ensures consistent ordering)
fn get_pool_key(token_a: &str, token_b: &str) -> (String, String) {
    if token_a < token_b {
        (token_a.to_string(), token_b.to_string())
    } else {
        (token_b.to_string(), token_a.to_string())
    }
}

/// Get or create pool
fn get_or_create_pool(state: &mut DEXState, token_a: &str, token_b: &str) -> &mut LiquidityPool {
    let key = get_pool_key(token_a, token_b);
    state.pools.entry(key).or_insert_with(LiquidityPool::new)
}

/// Get pool (read-only)
pub fn get_pool(state: &DEXState, token_a: &str, token_b: &str) -> Option<&LiquidityPool> {
    let key = get_pool_key(token_a, token_b);
    state.pools.get(&key)
}

/// Get LP shares for user
pub fn get_lp_shares(state: &DEXState, user: &str, token_a: &str, token_b: &str) -> u64 {
    let key = get_pool_key(token_a, token_b);
    state
        .lp_balances
        .get(user)
        .and_then(|pools| pools.get(&key))
        .copied()
        .unwrap_or(0)
}

/// Add liquidity to pool
pub fn add_liquidity(
    state: &mut DEXState,
    caller: &str,
    token_a: &str,
    token_b: &str,
    amount_a: u64,
    amount_b: u64,
) -> Result<u64, String> {
    if amount_a == 0 || amount_b == 0 {
        return Err("Cannot add zero liquidity".to_string());
    }

    let pool = get_or_create_pool(state, token_a, token_b);

    let shares = if pool.total_shares == 0 {
        // First liquidity provider
        // Shares = sqrt(amount_a * amount_b)
        let product = (amount_a as u128) * (amount_b as u128);
        (product as f64).sqrt() as u64
    } else {
        // Subsequent liquidity providers
        // Shares proportional to existing pool
        let share_a = (amount_a as u128 * pool.total_shares as u128) / pool.reserve_a as u128;
        let share_b = (amount_b as u128 * pool.total_shares as u128) / pool.reserve_b as u128;
        std::cmp::min(share_a, share_b) as u64
    };

    if shares == 0 {
        return Err("Insufficient liquidity minted".to_string());
    }

    // Update pool reserves
    pool.reserve_a += amount_a;
    pool.reserve_b += amount_b;
    pool.total_shares += shares;

    // Update user LP balance
    let key = get_pool_key(token_a, token_b);
    let user_shares = state
        .lp_balances
        .entry(caller.to_string())
        .or_insert_with(HashMap::new)
        .entry(key)
        .or_insert(0);
    *user_shares += shares;

    println!(
        "AddLiquidity: {} added {} {}, {} {} -> {} shares",
        caller, amount_a, token_a, amount_b, token_b, shares
    );

    Ok(shares)
}

/// Remove liquidity from pool
pub fn remove_liquidity(
    state: &mut DEXState,
    caller: &str,
    token_a: &str,
    token_b: &str,
    shares: u64,
) -> Result<(u64, u64), String> {
    if shares == 0 {
        return Err("Cannot remove zero shares".to_string());
    }

    let key = get_pool_key(token_a, token_b);

    // Check user has enough shares
    let user_shares = get_lp_shares(state, caller, token_a, token_b);
    if user_shares < shares {
        return Err(format!("Insufficient shares: {} < {}", user_shares, shares));
    }

    let pool = state
        .pools
        .get_mut(&key)
        .ok_or_else(|| "Pool does not exist".to_string())?;

    // Calculate amounts to return
    let amount_a = (shares as u128 * pool.reserve_a as u128) / pool.total_shares as u128;
    let amount_b = (shares as u128 * pool.reserve_b as u128) / pool.total_shares as u128;

    let amount_a = amount_a as u64;
    let amount_b = amount_b as u64;

    // Update pool
    pool.reserve_a -= amount_a;
    pool.reserve_b -= amount_b;
    pool.total_shares -= shares;

    // Update user shares
    let user_balance = state
        .lp_balances
        .get_mut(caller)
        .unwrap()
        .get_mut(&key)
        .unwrap();
    *user_balance -= shares;

    println!(
        "RemoveLiquidity: {} removed {} shares -> {} {}, {} {}",
        caller, shares, amount_a, token_a, amount_b, token_b
    );

    Ok((amount_a, amount_b))
}

/// Calculate output amount for swap (with fee)
pub fn get_amount_out(
    reserve_in: u64,
    reserve_out: u64,
    amount_in: u64,
    fee_basis_points: u64,
) -> u64 {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return 0;
    }

    // Apply fee: amount_in_with_fee = amount_in * (10000 - fee) / 10000
    let fee_multiplier = 10000 - fee_basis_points;
    let amount_in_with_fee = (amount_in as u128 * fee_multiplier as u128) / 10000;

    // Constant product formula: (x + dx) * (y - dy) = x * y
    // dy = (y * dx) / (x + dx)
    let numerator = amount_in_with_fee * reserve_out as u128;
    let denominator = (reserve_in as u128) + amount_in_with_fee;

    (numerator / denominator) as u64
}

/// Swap tokens
pub fn swap(
    state: &mut DEXState,
    caller: &str,
    token_in: &str,
    token_out: &str,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<u64, String> {
    if amount_in == 0 {
        return Err("Cannot swap zero amount".to_string());
    }

    let key = get_pool_key(token_in, token_out);
    let pool = state
        .pools
        .get_mut(&key)
        .ok_or_else(|| "Pool does not exist".to_string())?;

    // Determine which reserve is which
    let (reserve_in, reserve_out) = if token_in < token_out {
        (pool.reserve_a, pool.reserve_b)
    } else {
        (pool.reserve_b, pool.reserve_a)
    };

    // Calculate output amount
    let amount_out = get_amount_out(reserve_in, reserve_out, amount_in, state.fee_basis_points);

    // Slippage check
    if amount_out < min_amount_out {
        return Err(format!(
            "Slippage too high: {} < {}",
            amount_out, min_amount_out
        ));
    }

    // Update reserves
    if token_in < token_out {
        pool.reserve_a += amount_in;
        pool.reserve_b -= amount_out;
    } else {
        pool.reserve_b += amount_in;
        pool.reserve_a -= amount_out;
    }

    println!(
        "Swap: {} swapped {} {} for {} {}",
        caller, amount_in, token_in, amount_out, token_out
    );

    Ok(amount_out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        let state = initialize("owner".to_string(), 30); // 0.3% fee
        assert_eq!(state.owner, "owner");
        assert_eq!(state.fee_basis_points, 30);
    }

    #[test]
    fn test_add_liquidity_initial() {
        let mut state = initialize("owner".to_string(), 30);

        let shares = add_liquidity(&mut state, "alice", "TOKEN_A", "TOKEN_B", 1000, 1000).unwrap();

        assert!(shares > 0);
        assert_eq!(get_lp_shares(&state, "alice", "TOKEN_A", "TOKEN_B"), shares);

        let pool = get_pool(&state, "TOKEN_A", "TOKEN_B").unwrap();
        assert_eq!(pool.reserve_a, 1000);
        assert_eq!(pool.reserve_b, 1000);
    }

    #[test]
    fn test_add_and_remove_liquidity() {
        let mut state = initialize("owner".to_string(), 30);

        let shares = add_liquidity(&mut state, "alice", "TOKEN_A", "TOKEN_B", 1000, 2000).unwrap();

        let (amount_a, amount_b) =
            remove_liquidity(&mut state, "alice", "TOKEN_A", "TOKEN_B", shares).unwrap();

        assert_eq!(amount_a, 1000);
        assert_eq!(amount_b, 2000);
    }

    #[test]
    fn test_swap() {
        let mut state = initialize("owner".to_string(), 30);

        // Add initial liquidity
        add_liquidity(&mut state, "alice", "TOKEN_A", "TOKEN_B", 1000, 1000).unwrap();

        // Swap 100 TOKEN_A for TOKEN_B
        let amount_out = swap(&mut state, "bob", "TOKEN_A", "TOKEN_B", 100, 80).unwrap();

        assert!(amount_out >= 80); // Meets minimum
        assert!(amount_out < 100); // Due to slippage and fees

        let pool = get_pool(&state, "TOKEN_A", "TOKEN_B").unwrap();
        assert_eq!(pool.reserve_a, 1100); // 1000 + 100
        assert_eq!(pool.reserve_b, 1000 - amount_out);
    }

    #[test]
    fn test_get_amount_out() {
        // 1% fee
        let amount_out = get_amount_out(1000, 1000, 100, 100);

        // With 1% fee, effective input = 99
        // Output = (1000 * 99) / (1000 + 99) ≈ 90
        assert!(amount_out < 100);
        assert!(amount_out > 0);
    }

    #[test]
    fn test_slippage_protection() {
        let mut state = initialize("owner".to_string(), 30);

        add_liquidity(&mut state, "alice", "TOKEN_A", "TOKEN_B", 1000, 1000).unwrap();

        // Try to swap with unrealistic min_amount_out
        let result = swap(&mut state, "bob", "TOKEN_A", "TOKEN_B", 100, 200);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Slippage"));
    }
}
