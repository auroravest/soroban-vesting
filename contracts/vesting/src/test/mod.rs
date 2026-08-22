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
#[should_panic(expected = "Contract is already initialized")]
fn test_initialize_twice_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(TokenVesting, ());
    let client = TokenVestingClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    client.initialize(&admin, &token);
    client.initialize(&admin, &token);
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
#[should_panic(expected = "Amount must be greater than zero")]
fn test_create_vesting_zero_amount_panics() {
    let (env, _contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    client.create_vesting(&creator, &beneficiary, &0, &500, &2_000);
}

#[test]
fn test_partial_claims_accumulate_to_total_amount() {
    let (env, contract_id, client) = setup();
    let creator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let token_addr: Address = env.as_contract(&contract_id, || {
        env.storage().instance().get(&DataKey::Token).unwrap()
    });
    let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &token_addr);
    let token_client = soroban_sdk::token::Client::new(&env, &token_addr);

    token_admin.mint(&creator, &1_000);
    set_ledger_time(&env, 1_000);
    let id = client.create_vesting(&creator, &beneficiary, &1_000, &0, &1_000);

    for step in 1_u64..=10 {
        set_ledger_time(&env, 1_000 + step * 100);

        assert_eq!(client.claim(&beneficiary, &id), 100);

        let expected_claimed = i128::from(step) * 100;
        let info = client.get_vesting(&id);
        assert_eq!(info.schedule.claimed_amount, expected_claimed);
        assert_eq!(info.vested, expected_claimed);
        assert_eq!(info.claimable, 0);
        assert_eq!(token_client.balance(&beneficiary), expected_claimed);
        assert_eq!(token_client.balance(&contract_id), 1_000 - expected_claimed);
    }

    let final_info = client.get_vesting(&id);
    assert_eq!(final_info.schedule.claimed_amount, 1_000);
    assert_eq!(final_info.vested, 1_000);
    assert_eq!(final_info.claimable, 0);
}
