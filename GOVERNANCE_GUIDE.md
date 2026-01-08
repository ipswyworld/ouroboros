# Ouroboros Governance System

Complete guide to the Ouroboros on-chain governance system.

## Table of Contents

1. [Overview](#overview)
2. [Timelock System](#timelock-system)
3. [Emergency Pause](#emergency-pause)
4. [Proposals & Voting](#proposals--voting)
5. [Integration Guide](#integration-guide)
6. [API Reference](#api-reference)

---

## Overview

Ouroboros features a comprehensive governance system with:

- **Multisig Timelock**: 7-day delay for critical operations with 3-of-5 approval
- **Emergency Pause**: Guardian-based system to halt operations during crises
- **Proposal System**: On-chain proposals with token-weighted voting
- **Voting Mechanism**: Snapshot-based voting with quorum requirements

### Configuration

Default settings (can be customized):

```rust
pub struct GovernanceConfig {
    timelock_delay_secs: 7 * 24 * 60 * 60,  // 7 days
    min_guardians_for_pause: 3,              // 3 of 5
    total_guardians: 5,
    min_voting_period: 100_800,              // ~7 days at 6s/block
    quorum_percentage: 40,                   // 40% participation required
    proposal_threshold: 10_000_000_000,      // 10,000 OURO to create proposal
}
```

---

## Timelock System

The timelock controller ensures critical operations have a mandatory delay period for review.

### Key Features

- **7-day delay** before execution
- **3-of-5 multisig** approval required
- **Cancellable** before execution
- **Operation types** supported:
  - Parameter updates
  - Treasury transfers
  - Validator updates
  - Contract upgrades
  - Governance changes

### Usage Example

```rust
use ouro_dag::governance::{TimelockController, TimelockConfig, OperationType};

// Initialize timelock
let config = TimelockConfig {
    delay_secs: 7 * 24 * 60 * 60,
    admin_addresses: vec![
        "admin1".to_string(),
        "admin2".to_string(),
        "admin3".to_string(),
    ],
};

let mut timelock = TimelockController::new(config);

// Schedule operation
let operation = OperationType::UpdateParameter {
    key: "max_block_size".to_string(),
    value: "2000000".to_string(),
};

let op_id = timelock.schedule(
    operation,
    "admin1",
    "Increase max block size to 2MB".to_string(),
)?;

println!("Operation scheduled: {}", op_id);

// Add approvals (need 3 of 5)
timelock.approve(&op_id, "approver1", "signature1")?;
timelock.approve(&op_id, "approver2", "signature2")?;
timelock.approve(&op_id, "approver3", "signature3")?;

// Wait 7 days...

// Execute operation
let executed_op = timelock.execute(&op_id)?;
println!("Executed: {:?}", executed_op);
```

### Operation Types

#### 1. Update Parameter

```rust
OperationType::UpdateParameter {
    key: "max_tx_size".to_string(),
    value: "1000000".to_string(),
}
```

#### 2. Transfer Treasury

```rust
OperationType::TransferTreasury {
    to: "recipient_address".to_string(),
    amount: 1_000_000_000, // 10 OURO
}
```

#### 3. Update Validators

```rust
OperationType::UpdateValidators {
    add: vec!["new_validator1".to_string()],
    remove: vec!["old_validator1".to_string()],
}
```

#### 4. Upgrade Contract

```rust
OperationType::UpgradeContract {
    contract_address: "0x123...".to_string(),
    new_code_hash: "0xabc...".to_string(),
}
```

### Cancellation

```rust
// Cancel before execution
timelock.cancel(&op_id, "admin1")?;
```

---

## Emergency Pause

Guardian-based emergency system to pause operations during critical situations.

### Key Features

- **3-of-5 guardians** required to activate
- **Auto-expires** after 24 hours
- **Reversible** with guardian consensus
- **Multiple pause reasons** supported

### Usage Example

```rust
use ouro_dag::governance::{EmergencyPause, GuardianSet, PauseReason};

// Initialize guardians
let guardians = GuardianSet::new(
    vec![
        "guardian1".to_string(),
        "guardian2".to_string(),
        "guardian3".to_string(),
        "guardian4".to_string(),
        "guardian5".to_string(),
    ],
    3, // 3 of 5 required
);

let mut pause = EmergencyPause::new(guardians);

// Guardian votes to pause
let reason = PauseReason::SecurityVulnerability {
    description: "Critical bug in contract X".to_string(),
};

// First vote
pause.vote_pause("guardian1", "sig1", reason.clone())?;
// Second vote
pause.vote_pause("guardian2", "sig2", reason.clone())?;
// Third vote - ACTIVATES PAUSE
let activated = pause.vote_pause("guardian3", "sig3", reason)?;

if activated {
    println!("🚨 EMERGENCY PAUSE ACTIVATED");
}

// Check if paused
if pause.is_paused() {
    return Err("System is paused");
}

// ... fix the issue ...

// Vote to unpause (all 3 must vote to unpause)
pause.vote_unpause("guardian1", "sig1", "Issue fixed".to_string())?;
pause.vote_unpause("guardian2", "sig2", "Issue fixed".to_string())?;
let unpaused = pause.vote_unpause("guardian3", "sig3", "Issue fixed".to_string())?;

if unpaused {
    println!("✅ System unpaused");
}
```

### Pause Reasons

```rust
pub enum PauseReason {
    SecurityVulnerability { description: String },
    ActiveExploit { description: String },
    OracleManipulation { oracle_id: String },
    ConsensusFailure { description: String },
    NetworkPartition { description: String },
    Other { reason: String },
}
```

### Integration with Operations

```rust
// Check before critical operations
if pause.is_paused() {
    return Err("Cannot process: system is paused");
}

// Proceed with operation
process_transaction(tx)?;
```

---

## Proposals & Voting

On-chain governance with token-weighted voting.

### Key Features

- **Token-weighted**: 1 OURO = 1 vote
- **Snapshot-based**: Balance frozen at proposal creation
- **Quorum requirement**: Default 40% participation
- **Voting period**: Minimum 7 days (100,800 blocks)

### Proposal Lifecycle

```
Active → Voting Period → Finalized → Executed
  ↓                          ↓
Cancelled                Rejected
```

### Usage Example

```rust
use ouro_dag::governance::{
    ProposalRegistry, ProposalType, VotingRegistry, VoteChoice
};
use std::collections::HashMap;

// Initialize registries
let mut proposals = ProposalRegistry::new();
let mut voting = VotingRegistry::new(100_800, 40); // 7 days, 40% quorum

// Create proposal
let proposal_id = proposals.create_proposal(
    ProposalType::ParameterChange {
        parameter: "max_block_size".to_string(),
        current_value: "1000000".to_string(),
        new_value: "2000000".to_string(),
    },
    "proposer_address".to_string(),
    "Increase max block size to improve throughput".to_string(),
    current_block,
    None, // Use default voting period
    total_voting_power,
)?;

// Create voting snapshot
let mut balances = HashMap::new();
balances.insert("voter1".to_string(), 100_000_000_000); // 1,000 OURO
balances.insert("voter2".to_string(), 500_000_000_000); // 5,000 OURO
balances.insert("voter3".to_string(), 200_000_000_000); // 2,000 OURO

voting.create_snapshot(proposal_id.clone(), current_block, balances);

// Cast votes
voting.cast_vote(
    proposal_id.clone(),
    "voter1".to_string(),
    VoteChoice::Yes,
    current_block + 100,
    "signature1".to_string(),
)?;

voting.cast_vote(
    proposal_id.clone(),
    "voter2".to_string(),
    VoteChoice::Yes,
    current_block + 200,
    "signature2".to_string(),
)?;

voting.cast_vote(
    proposal_id.clone(),
    "voter3".to_string(),
    VoteChoice::No,
    current_block + 300,
    "signature3".to_string(),
)?;

// After voting period ends...
let (yes, no, abstain) = voting.tally_votes(&proposal_id)?;
println!("Results: Yes: {}, No: {}, Abstain: {}", yes, no, abstain);

// Finalize proposal
let proposal = proposals.get_proposal_mut(&proposal_id).unwrap();
proposal.yes_votes = yes;
proposal.no_votes = no;
proposal.abstain_votes = abstain;
proposal.finalize(current_block + 100_800, 40)?;

// Execute if passed
if proposal.status == ProposalStatus::Passed {
    // Execute the proposal
    execute_parameter_change(&proposal.proposal_type)?;
    proposal.mark_executed("tx_hash".to_string());
}
```

### Proposal Types

#### 1. Parameter Change

```rust
ProposalType::ParameterChange {
    parameter: "fee_percentage".to_string(),
    current_value: "0.1".to_string(),
    new_value: "0.05".to_string(),
}
```

#### 2. Treasury Spend

```rust
ProposalType::TreasurySpend {
    recipient: "dev_team_address".to_string(),
    amount: 100_000_000_000, // 1,000 OURO
    purpose: "Development grant Q1 2024".to_string(),
}
```

#### 3. Validator Update

```rust
ProposalType::ValidatorUpdate {
    add: vec!["new_validator".to_string()],
    remove: vec!["old_validator".to_string()],
}
```

#### 4. Contract Upgrade

```rust
ProposalType::ContractUpgrade {
    contract_address: "0x123...".to_string(),
    new_code_hash: "0xabc...".to_string(),
}
```

#### 5. Governance Change

```rust
ProposalType::GovernanceChange {
    change_type: "quorum".to_string(),
    details: "Update quorum from 40% to 30%".to_string(),
}
```

#### 6. General Proposal

```rust
ProposalType::General {
    title: "Community Initiative".to_string(),
    description: "Proposal to support ecosystem growth".to_string(),
}
```

### Voting

```rust
// Vote YES
voting.cast_vote(
    proposal_id.clone(),
    voter_address,
    VoteChoice::Yes,
    current_block,
    signature,
)?;

// Vote NO
voting.cast_vote(
    proposal_id.clone(),
    voter_address,
    VoteChoice::No,
    current_block,
    signature,
)?;

// Abstain
voting.cast_vote(
    proposal_id.clone(),
    voter_address,
    VoteChoice::Abstain,
    current_block,
    signature,
)?;
```

### Quorum & Passing

```rust
// Check quorum
let has_quorum = voting.has_quorum(&proposal_id)?;

// Check if passed (yes > no + quorum met)
let proposal = proposals.get_proposal(&proposal_id).unwrap();
let passed = proposal.has_passed(40); // 40% quorum
```

---

## Integration Guide

### Step 1: Initialize Governance

```rust
use ouro_dag::governance::{GovernanceController, GovernanceConfig};

// Create guardians list
let guardians = vec![
    "guardian1_address".to_string(),
    "guardian2_address".to_string(),
    "guardian3_address".to_string(),
    "guardian4_address".to_string(),
    "guardian5_address".to_string(),
];

// Initialize governance
let governance = Arc::new(RwLock::new(
    GovernanceController::new(
        GovernanceConfig::default(),
        guardians,
    )
));
```

### Step 2: Add to Node Startup

```rust
// In lib.rs startup sequence
use ouro_dag::governance::{GovernanceController, GovernanceConfig, GovernanceIntegration};

// After other initializations...
let governance = Arc::new(RwLock::new(
    GovernanceController::new(config, guardians)
));

let gov_integration = GovernanceIntegration::new(governance.clone());

println!("🏛️  Governance system initialized");

// Start proposal finalization task
let (block_tx, block_rx) = tokio::sync::watch::channel(0u64);
tokio::spawn(finalize_ended_proposals_task(
    governance.clone(),
    block_rx,
));
```

### Step 3: Check Pause Before Operations

```rust
// Before processing transactions
gov_integration.check_not_paused().await?;

// Process transaction
process_transaction(tx)?;
```

### Step 4: Environment Configuration

Add to `.env`:

```bash
# Governance
GOVERNANCE_ENABLED=true
GUARDIAN_ADDRESSES=addr1,addr2,addr3,addr4,addr5
MIN_GUARDIANS_FOR_PAUSE=3
TIMELOCK_DELAY_DAYS=7
PROPOSAL_THRESHOLD=10000000000  # 10,000 OURO
QUORUM_PERCENTAGE=40
```

### Step 5: API Endpoints

```rust
// Add governance routes
async fn create_proposal_endpoint(
    Json(params): Json<CreateProposalRequest>,
) -> Result<Json<String>, StatusCode> {
    let proposal_id = gov_integration.create_proposal(
        params.proposal_type,
        params.proposer,
        params.description,
        current_block,
        total_voting_power,
    ).await?;

    Ok(Json(proposal_id))
}

async fn vote_endpoint(
    Json(params): Json<VoteRequest>,
) -> Result<Json<u64>, StatusCode> {
    let voting_power = gov_integration.vote_on_proposal(
        params.proposal_id,
        params.voter,
        params.choice,
        current_block,
        params.signature,
    ).await?;

    Ok(Json(voting_power))
}
```

---

## API Reference

### GovernanceController

```rust
pub struct GovernanceController {
    pub fn new(config: GovernanceConfig, guardians: Vec<String>) -> Self;
    pub async fn is_paused(&self) -> bool;
    pub fn timelock(&self) -> Arc<RwLock<TimelockController>>;
    pub fn pause(&self) -> Arc<RwLock<EmergencyPause>>;
    pub fn proposals(&self) -> Arc<RwLock<ProposalRegistry>>;
    pub fn voting(&self) -> Arc<RwLock<VotingRegistry>>;
}
```

### TimelockController

```rust
pub struct TimelockController {
    pub fn new(config: TimelockConfig) -> Self;
    pub fn schedule(&mut self, operation: OperationType, proposer: &str, description: String) -> Result<String, String>;
    pub fn approve(&mut self, operation_id: &str, approver: &str, signature: &str) -> Result<(), String>;
    pub fn execute(&mut self, operation_id: &str) -> Result<OperationType, String>;
    pub fn cancel(&mut self, operation_id: &str, canceller: &str) -> Result<(), String>;
    pub fn get_operation(&self, operation_id: &str) -> Option<&TimelockOperation>;
}
```

### EmergencyPause

```rust
pub struct EmergencyPause {
    pub fn new(guardian_set: GuardianSet) -> Self;
    pub fn is_paused(&self) -> bool;
    pub fn vote_pause(&mut self, guardian: &str, signature: &str, reason: PauseReason) -> Result<bool, String>;
    pub fn vote_unpause(&mut self, guardian: &str, signature: &str, resolution: String) -> Result<bool, String>;
    pub fn get_state(&self) -> &PauseState;
}
```

### ProposalRegistry

```rust
pub struct ProposalRegistry {
    pub fn new() -> Self;
    pub fn create_proposal(&mut self, proposal_type: ProposalType, proposer: String, description: String, current_block: u64, voting_period_blocks: Option<u64>, total_voting_power: u64) -> Result<String, String>;
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal>;
    pub fn get_active_proposals(&self, current_block: u64) -> Vec<&Proposal>;
    pub fn cancel_proposal(&mut self, proposal_id: &str, canceller: &str) -> Result<(), String>;
}
```

### VotingRegistry

```rust
pub struct VotingRegistry {
    pub fn new(min_voting_period: u64, quorum_percentage: u8) -> Self;
    pub fn create_snapshot(&mut self, proposal_id: String, snapshot_block: u64, balances: HashMap<String, u64>) -> u64;
    pub fn cast_vote(&mut self, proposal_id: String, voter: String, choice: VoteChoice, current_block: u64, signature: String) -> Result<u64, String>;
    pub fn tally_votes(&self, proposal_id: &str) -> Result<(u64, u64, u64), String>;
    pub fn has_quorum(&self, proposal_id: &str) -> Result<bool, String>;
}
```

---

## Best Practices

### Security

1. **Always verify signatures** for votes and guardian actions
2. **Use timelock** for all critical parameter changes
3. **Set appropriate quorum** to prevent governance attacks
4. **Regular guardian rotation** for decentralization

### Gas Optimization

1. **Batch vote counting** instead of counting on each vote
2. **Use snapshots** to avoid iterating balances
3. **Limit proposal descriptions** to reasonable sizes

### Testing

```rust
#[tokio::test]
async fn test_full_governance_flow() {
    // 1. Create proposal
    // 2. Create voting snapshot
    // 3. Cast votes
    // 4. Finalize proposal
    // 5. Execute if passed
}
```

---

## Examples

### Example 1: Parameter Update via Governance

```rust
// 1. Create proposal
let proposal_id = proposals.create_proposal(
    ProposalType::ParameterChange {
        parameter: "tx_fee".to_string(),
        current_value: "1000".to_string(),
        new_value: "500".to_string(),
    },
    "community_member".to_string(),
    "Reduce transaction fees to increase adoption".to_string(),
    1000,
    None,
    total_supply,
)?;

// 2. Community votes for 7 days

// 3. Proposal passes, schedule in timelock
let mut timelock = timelock_controller.lock().await;
let op_id = timelock.schedule(
    OperationType::UpdateParameter {
        key: "tx_fee".to_string(),
        value: "500".to_string(),
    },
    "governance",
    format!("Executing proposal {}", proposal_id),
)?;

// 4. Wait 7 days, get approvals, execute
```

### Example 2: Emergency Pause Due to Exploit

```rust
// Guardian detects exploit
let reason = PauseReason::ActiveExploit {
    description: "Reentrancy attack on contract X".to_string(),
};

// 3 guardians vote to pause
pause.vote_pause("guardian1", "sig1", reason.clone())?;
pause.vote_pause("guardian2", "sig2", reason.clone())?;
let activated = pause.vote_pause("guardian3", "sig3", reason)?;

// System is now paused
// Fix the issue...
// Deploy fix...

// Unpause after fix
pause.vote_unpause("guardian1", "sig1", "Exploit patched in v1.2.1".to_string())?;
pause.vote_unpause("guardian2", "sig2", "Exploit patched in v1.2.1".to_string())?;
pause.vote_unpause("guardian3", "sig3", "Exploit patched in v1.2.1".to_string())?;
```

---

## Resources

- **Source Code**: `ouro_dag/src/governance/`
- **Tests**: Each module includes comprehensive tests
- **Integration**: `ouro_dag/src/governance/integration.rs`

---

## License

MIT License - see LICENSE file for details
