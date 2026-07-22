//! Tests for TokenVesting contract
//!
//! Covers initialization, vesting creation, claiming, revocation,
//! edge cases, and authorization boundaries.

extern crate std;

use crate::{DataKey, TokenVesting, TokenVestingClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};
use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe},
    string::{String, ToString},
};

fn panic_text(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }

    std::panic::panic_any("panic payload was not a string");
}

fn assert_panics_with(expected: &str, action: impl FnOnce()) {
    let result = catch_unwind(AssertUnwindSafe(action));
    let payload = result.expect_err("expected operation to panic");
    let host_message = panic_text(payload.as_ref());
    let message = host_message
        .split_once("caught panic '")
        .and_then(|(_, rest)| rest.split_once("' from contract function"))
        .map(|(message, _)| message)
        .expect("Soroban host panic did not contain a contract panic message");
    assert_eq!(message, expected);
}

fn setup() -> (Env, Address, TokenVestingClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TokenVesting, ());
    let client = TokenVestingClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    client.initialize(&admin, &token);
    (env, contract_id, client)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TokenVesting, ());
    let client = TokenVestingClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    client.initialize(&admin, &token);
    assert_eq!(client.get_vesting_count(), 0);
}

#[test]
fn test_initialize_twice_panics_with_exact_message() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TokenVesting, ());
    let client = TokenVestingClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    client.initialize(&admin, &token);
    assert_panics_with("Contract is already initialized", || {
        client.initialize(&admin, &token);
    });
}

#[test]
fn test_create_vesting() {
    let (env, contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token_addr: Address = env.as_contract(&contract_id, || {
        env.storage().instance().get(&DataKey::Token).unwrap()
    });
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_addr);
    token_admin.mint(&creator, &10_000);
    set_ledger_time(&env, 1_000);

    let id = client.create_vesting(&creator, &beneficiary, &1_000, &500, &2_000);
    assert_eq!(id, 0);
    assert_eq!(client.get_vesting_count(), 1);

    let info = client.get_vesting(&id);
    assert_eq!(info.schedule.total_amount, 1_000);
    assert_eq!(info.schedule.beneficiary, beneficiary);
    assert_eq!(info.schedule.claimed_amount, 0);
}

fn set_ledger_time(env: &Env, timestamp: u64) {
    env.ledger().set_timestamp(timestamp);
}

#[test]
fn test_uninitialized_query_panics_with_exact_message() {
    let env = Env::default();
    let contract_id = env.register(TokenVesting, ());
    let client = TokenVestingClient::new(&env, &contract_id);

    assert_panics_with("Contract is not initialized", || {
        client.get_vesting_count();
    });
}

#[test]
fn test_create_vesting_zero_amount_panics_with_exact_message() {
    let (env, _contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    assert_panics_with("Amount must be greater than zero", || {
        client.create_vesting(&creator, &beneficiary, &0, &500, &2_000);
    });
}

#[test]
fn test_create_vesting_zero_duration_panics_with_exact_message() {
    let (env, _contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    assert_panics_with("Vesting duration must be greater than zero", || {
        client.create_vesting(&creator, &beneficiary, &1, &500, &0);
    });
}

#[test]
fn test_create_vesting_timestamp_overflow_panics_with_exact_message() {
    let (env, _contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    set_ledger_time(&env, u64::MAX);

    assert_panics_with("overflow", || {
        client.create_vesting(&creator, &beneficiary, &1, &1, &1);
    });
}

fn create_funded_vesting(
    env: &Env,
    contract_id: &Address,
    client: &TokenVestingClient<'_>,
    creator: &Address,
    beneficiary: &Address,
) -> u64 {
    let token_addr: Address = env.as_contract(contract_id, || {
        env.storage().instance().get(&DataKey::Token).unwrap()
    });
    soroban_sdk::token::StellarAssetClient::new(env, &token_addr).mint(creator, &10_000);
    client.create_vesting(creator, beneficiary, &1_000, &500, &2_000)
}

#[test]
fn test_missing_schedule_panics_with_exact_message_for_each_query() {
    let (_env, _contract_id, client) = setup();

    assert_panics_with("Vesting schedule not found", || {
        client.get_vesting(&404);
    });
    assert_panics_with("Vesting schedule not found", || {
        client.get_claimable(&404);
    });
}

#[test]
fn test_claim_missing_schedule_panics_with_exact_message() {
    let (env, _contract_id, client) = setup();
    let beneficiary = Address::generate(&env);

    assert_panics_with("Vesting schedule not found", || {
        client.claim(&beneficiary, &404);
    });
}

#[test]
fn test_revoke_missing_schedule_panics_with_exact_message() {
    let (env, _contract_id, client) = setup();
    let creator = Address::generate(&env);

    assert_panics_with("Vesting schedule not found", || {
        client.revoke(&creator, &404);
    });
}

#[test]
fn test_claim_by_wrong_beneficiary_panics_with_exact_message() {
    let (env, contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let stranger = Address::generate(&env);
    let id = create_funded_vesting(&env, &contract_id, &client, &creator, &beneficiary);

    assert_panics_with("Not the beneficiary", || {
        client.claim(&stranger, &id);
    });
}

#[test]
fn test_claim_before_cliff_panics_with_exact_message() {
    let (env, contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let id = create_funded_vesting(&env, &contract_id, &client, &creator, &beneficiary);

    assert_panics_with("Nothing to claim", || {
        client.claim(&beneficiary, &id);
    });
}

#[test]
fn test_claim_revoked_schedule_panics_with_exact_message() {
    let (env, contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let id = create_funded_vesting(&env, &contract_id, &client, &creator, &beneficiary);
    client.revoke(&creator, &id);

    assert_panics_with("Vesting has been revoked", || {
        client.claim(&beneficiary, &id);
    });
}

#[test]
fn test_revoke_by_wrong_creator_panics_with_exact_message() {
    let (env, contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let stranger = Address::generate(&env);
    let id = create_funded_vesting(&env, &contract_id, &client, &creator, &beneficiary);

    assert_panics_with("Not the creator", || {
        client.revoke(&stranger, &id);
    });
}

#[test]
fn test_revoke_twice_panics_with_exact_message() {
    let (env, contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let id = create_funded_vesting(&env, &contract_id, &client, &creator, &beneficiary);
    client.revoke(&creator, &id);

    assert_panics_with("Already revoked", || {
        client.revoke(&creator, &id);
    });
}
