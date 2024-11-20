use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use btb_finance_clmm::{
    fees::{collect_fees, CollectFeesAccounts, Position},
    utils::{calc_fees_earned, price_to_tick},
};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_collect_fees() {
    // Initialize program test environment
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "btb_finance_clmm",
        program_id,
        processor!(btb_finance_clmm::entry),
    );

    // Create test accounts
    let owner = Keypair::new();
    let pool = Keypair::new();
    let position = Keypair::new();
    let token_a_mint = Keypair::new();
    let token_b_mint = Keypair::new();
    let token_account_a = Keypair::new();
    let token_account_b = Keypair::new();
    let pool_token_a = Keypair::new();
    let pool_token_b = Keypair::new();

    // Add test accounts to program
    program_test.add_account(
        owner.pubkey(),
        solana_sdk::account::Account {
            lamports: 1_000_000_000,
            data: vec![],
            owner: solana_program::system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    // Start program test
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create position with initial state
    let position_data = Position {
        owner: owner.pubkey(),
        pool: pool.pubkey(),
        tick_lower: price_to_tick(900_000).unwrap(), // $9.00
        tick_upper: price_to_tick(1_100_000).unwrap(), // $11.00
        liquidity: 1_000_000,
        fee_growth_inside_a: 0,
        fee_growth_inside_b: 0,
        tokens_owed_a: 0,
        tokens_owed_b: 0,
    };

    // Create collect fees instruction
    let accounts = CollectFeesAccounts {
        owner: owner.pubkey(),
        pool: pool.pubkey(),
        position: position.pubkey(),
        token_account_a: token_account_a.pubkey(),
        token_account_b: token_account_b.pubkey(),
        pool_token_a: pool_token_a.pubkey(),
        pool_token_b: pool_token_b.pubkey(),
        token_program: token::ID,
    };

    // Test collecting fees
    let result = collect_fees(Context::new(
        &program_id,
        &accounts,
        &[&owner],
        Some(recent_blockhash),
    ));

    assert!(result.is_ok(), "Fee collection should succeed");

    // Test fee calculation
    let fees_earned = calc_fees_earned(
        position_data.liquidity,
        position_data.fee_growth_inside_a,
        1_000_000, // Example global fee growth
    )
    .unwrap();

    assert!(fees_earned > 0, "Should have earned fees");
}

#[tokio::test]
async fn test_fee_calculation() {
    // Test various fee scenarios
    let test_cases = vec![
        // (liquidity, fee_growth_inside, fee_growth_global, expected_fees)
        (1_000_000, 0, 1_000_000, 1_000_000),
        (500_000, 500_000, 1_000_000, 250_000),
        (2_000_000, 800_000, 1_000_000, 400_000),
    ];

    for (liquidity, fee_growth_inside, fee_growth_global, expected_fees) in test_cases {
        let fees = calc_fees_earned(liquidity, fee_growth_inside, fee_growth_global).unwrap();
        assert_eq!(
            fees, expected_fees,
            "Fee calculation failed for liquidity={}, inside={}, global={}",
            liquidity, fee_growth_inside, fee_growth_global
        );
    }
}

#[tokio::test]
async fn test_position_validation() {
    // Test invalid position scenarios
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "btb_finance_clmm",
        program_id,
        processor!(btb_finance_clmm::entry),
    );

    let owner = Keypair::new();
    let wrong_owner = Keypair::new();
    let pool = Keypair::new();
    let position = Keypair::new();

    // Create position with wrong owner
    let position_data = Position {
        owner: wrong_owner.pubkey(),
        pool: pool.pubkey(),
        tick_lower: price_to_tick(900_000).unwrap(),
        tick_upper: price_to_tick(1_100_000).unwrap(),
        liquidity: 1_000_000,
        fee_growth_inside_a: 0,
        fee_growth_inside_b: 0,
        tokens_owed_a: 0,
        tokens_owed_b: 0,
    };

    // Test collecting fees with wrong owner
    let accounts = CollectFeesAccounts {
        owner: owner.pubkey(),
        pool: pool.pubkey(),
        position: position.pubkey(),
        token_account_a: Pubkey::new_unique(),
        token_account_b: Pubkey::new_unique(),
        pool_token_a: Pubkey::new_unique(),
        pool_token_b: Pubkey::new_unique(),
        token_program: token::ID,
    };

    let result = collect_fees(Context::new(
        &program_id,
        &accounts,
        &[&owner],
        None,
    ));

    assert!(result.is_err(), "Should fail with invalid owner");
}
