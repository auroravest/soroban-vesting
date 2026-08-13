//! Token Vesting — Soroban Smart Contract
//!
//! On-chain token vesting with cliff and linear unlock schedules.
//! Supports multiple vesting schedules per contract, revocable by creator.

#![no_std]
extern crate alloc;
#[cfg(test)]
extern crate std;

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

// --- Storage keys ---

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Initialized,
    VestingCount,
    Vesting(u64),
}

// --- Types ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub id: u64,
    pub creator: Address,
    pub beneficiary: Address,
    pub total_amount: i128,
    pub claimed_amount: i128,
    pub revoked: bool,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfo {
    pub schedule: VestingSchedule,
    pub vested: i128,
    pub claimable: i128,
}

// --- Contract ---

#[contract]
pub struct TokenVesting;

#[contractimpl]
impl TokenVesting {
    // Helper: assert initialized
    fn require_init(env: &Env) {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic!("Contract is not initialized");
        }
    }

    // --- Initialization ---

    /// Initialize the contract with admin and token addresses.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Contract is already initialized");
        }
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::VestingCount, &0_u64);

        let topics = (Symbol::new(&env, "initialize"), admin);
        env.events().publish(topics, token);
    }

    // --- Vesting Creation ---

    /// Create a new linear vesting schedule with cliff.
    /// Transfers `amount` tokens from creator to the contract immediately.
    pub fn create_vesting(
        env: Env,
        creator: Address,
        beneficiary: Address,
        amount: i128,
        cliff_secs: u64,
        vest_secs: u64,
    ) -> u64 {
        Self::require_init(&env);
        creator.require_auth();

        if amount <= 0 {
            panic!("Amount must be greater than zero");
        }
        if vest_secs == 0 {
            panic!("Vesting duration must be greater than zero");
        }

        let now = env.ledger().timestamp();
        let cliff_time = now.checked_add(cliff_secs).expect("overflow");
        let end_time = cliff_time.checked_add(vest_secs).expect("overflow");

        // Transfer tokens from creator to contract
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&creator, &env.current_contract_address(), &amount);

        // Get next vesting ID
        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::VestingCount)
            .unwrap();
        let id = count;
        count = count.checked_add(1).expect("overflow");
        env.storage().instance().set(&DataKey::VestingCount, &count);

        let schedule = VestingSchedule {
            id,
            creator: creator.clone(),
            beneficiary: beneficiary.clone(),
            total_amount: amount,
            claimed_amount: 0,
            revoked: false,
            start_time: now,
            cliff_time,
            end_time,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Vesting(id), &schedule);

        let topics = (Symbol::new(&env, "create_vesting"), creator);
        env.events()
            .publish(topics, (id, beneficiary, amount, cliff_time, end_time));

        id
    }

    // --- Claim ---

    /// Claim vested tokens. Callable by the beneficiary.
    pub fn claim(env: Env, beneficiary: Address, vesting_id: u64) -> i128 {
        Self::require_init(&env);
        beneficiary.require_auth();

        let mut schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Vesting(vesting_id))
            .unwrap_or_else(|| panic!("Vesting schedule not found"));

        if schedule.beneficiary != beneficiary {
            panic!("Not the beneficiary");
        }
        if schedule.revoked {
            panic!("Vesting has been revoked");
        }

        let claimable = Self::calculate_claimable(&env, &schedule);
        if claimable <= 0 {
            panic!("Nothing to claim");
        }

        schedule.claimed_amount = schedule
            .claimed_amount
            .checked_add(claimable)
            .expect("overflow");
        env.storage()
            .persistent()
            .set(&DataKey::Vesting(vesting_id), &schedule);

        // Transfer tokens to beneficiary
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &beneficiary, &claimable);

        let topics = (Symbol::new(&env, "claim"), beneficiary);
        env.events().publish(topics, (vesting_id, claimable));

        claimable
    }

    // --- Revoke ---

    /// Revoke unvested tokens. Returns remaining tokens to creator.
    pub fn revoke(env: Env, creator: Address, vesting_id: u64) -> i128 {
        Self::require_init(&env);
        creator.require_auth();

        let mut schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Vesting(vesting_id))
            .unwrap_or_else(|| panic!("Vesting schedule not found"));

        if schedule.creator != creator {
            panic!("Not the creator");
        }
        if schedule.revoked {
            panic!("Already revoked");
        }

        schedule.revoked = true;
        let vested = Self::calculate_vested(&env, &schedule);
        let remaining = schedule
            .total_amount
            .checked_sub(vested)
            .expect("underflow");
        let unclaimed = remaining.checked_sub(schedule.claimed_amount).unwrap_or(0);

        env.storage()
            .persistent()
            .set(&DataKey::Vesting(vesting_id), &schedule);

        if unclaimed > 0 {
            let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
            let token_client = token::Client::new(&env, &token_addr);
            token_client.transfer(&env.current_contract_address(), &creator, &unclaimed);
        }

        let topics = (Symbol::new(&env, "revoke"), creator);
        env.events().publish(topics, (vesting_id, unclaimed));

        unclaimed
    }

    // --- Queries ---

    /// Calculate total vested amount (including already claimed).
    fn calculate_vested(env: &Env, schedule: &VestingSchedule) -> i128 {
        let now = env.ledger().timestamp();
        if now < schedule.cliff_time {
            return 0;
        }
        if now >= schedule.end_time {
            return schedule.total_amount;
        }
        let elapsed = (now - schedule.cliff_time) as i128;
        let duration = (schedule.end_time - schedule.cliff_time) as i128;
        if duration == 0 {
            return schedule.total_amount;
        }
        schedule.total_amount * elapsed / duration
    }

    /// Calculate currently claimable amount.
    fn calculate_claimable(env: &Env, schedule: &VestingSchedule) -> i128 {
        let vested = Self::calculate_vested(env, schedule);
        let remaining = vested.checked_sub(schedule.claimed_amount).unwrap_or(0);
        if remaining < 0 {
            0
        } else {
            remaining
        }
    }

    /// Get detailed vesting info including vested and claimable amounts.
    pub fn get_vesting(env: Env, vesting_id: u64) -> VestingInfo {
        Self::require_init(&env);
        let schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Vesting(vesting_id))
            .unwrap_or_else(|| panic!("Vesting schedule not found"));
        let vested = Self::calculate_vested(&env, &schedule);
        let claimable = Self::calculate_claimable(&env, &schedule);
        VestingInfo {
            schedule: schedule.clone(),
            vested,
            claimable,
        }
    }

    /// Get claimable amount without modifying state.
    pub fn get_claimable(env: Env, vesting_id: u64) -> i128 {
        Self::require_init(&env);
        let schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Vesting(vesting_id))
            .unwrap_or_else(|| panic!("Vesting schedule not found"));
        Self::calculate_claimable(&env, &schedule)
    }

    /// Get total number of vesting schedules.
    pub fn get_vesting_count(env: Env) -> u64 {
        Self::require_init(&env);
        env.storage()
            .instance()
            .get(&DataKey::VestingCount)
            .unwrap_or(0)
    }
}

// --- Tests ---

#[cfg(test)]
mod test;
