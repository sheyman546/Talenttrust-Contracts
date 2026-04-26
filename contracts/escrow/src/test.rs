use soroban_sdk::{symbol_short, testutils::{Address as _, Ledger, LedgerInfo}, vec, Address, Env};

use crate::{Escrow, EscrowClient};

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("World"));
    assert_eq!(result, symbol_short!("World"));
}

#[test]
fn test_create_contract() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 200_0000000_i128, 400_0000000_i128, 600_0000000_i128];

    let id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    assert_eq!(id, 1);
}

#[test]
fn test_deposit_funds() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let result = client.deposit_funds(&1, &1_000_0000000);
    assert!(result);
}

#[test]
fn test_release_milestone() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let result = client.release_milestone(&1, &0);
    assert!(result);
}

// ============================================================================
// Time Management Tests
// ============================================================================

#[test]
fn test_schedule_milestone() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Set initial ledger time to a known value
    env.ledger().set(LedgerInfo {
        timestamp: 1_000_000,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Schedule a milestone 7 days (604800 seconds) in the future
    let deadline = client.schedule_milestone(&604_800);
    
    // Verify the deadline is correctly calculated
    assert_eq!(deadline, 1_604_800);
}

#[test]
fn test_milestone_not_expired() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Set current time
    env.ledger().set(LedgerInfo {
        timestamp: 1_000_000,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Check a deadline in the future
    let deadline = 2_000_000;
    let is_expired = client.is_milestone_expired(&deadline);
    
    assert!(!is_expired, "Milestone should not be expired");
}

#[test]
fn test_milestone_expired() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Set current time
    env.ledger().set(LedgerInfo {
        timestamp: 2_000_000,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Check a deadline in the past
    let deadline = 1_000_000;
    let is_expired = client.is_milestone_expired(&deadline);
    
    assert!(is_expired, "Milestone should be expired");
}

#[test]
fn test_can_dispute_within_window() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Set current time
    env.ledger().set(LedgerInfo {
        timestamp: 1_000_000,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Dispute deadline is in the future
    let dispute_deadline = 1_500_000;
    let can_dispute = client.can_dispute(&dispute_deadline);
    
    assert!(can_dispute, "Should be able to dispute within window");
}

#[test]
fn test_cannot_dispute_after_window() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Set current time
    env.ledger().set(LedgerInfo {
        timestamp: 2_000_000,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    // Dispute deadline has passed
    let dispute_deadline = 1_500_000;
    let can_dispute = client.can_dispute(&dispute_deadline);
    
    assert!(!can_dispute, "Should not be able to dispute after window");
}

#[test]
fn test_time_advancement() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Set initial time
    env.ledger().set(LedgerInfo {
        timestamp: 1_000_000,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });

    let deadline = 1_500_000;
    
    // Initially not expired
    assert!(!client.is_milestone_expired(&deadline));
    
    // Advance time past the deadline
    env.ledger().set(LedgerInfo {
        timestamp: 1_600_000,
        protocol_version: 20,
        sequence_number: 11,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });
    
    // Now expired
    assert!(client.is_milestone_expired(&deadline));
}

#[test]
fn test_exact_deadline_boundary() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let deadline = 1_000_000;
    
    // Set time exactly at deadline
    env.ledger().set(LedgerInfo {
        timestamp: deadline,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 3110400,
    });
    
    // At exact deadline, milestone is NOT expired (> not >=)
    assert!(!client.is_milestone_expired(&deadline));
    
    // But dispute window is still open (<=)
    assert!(client.can_dispute(&deadline));
}
