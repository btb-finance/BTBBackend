use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use raydium_cpi::{state::*, utils::*, instructions::tick_array::*};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

pub mod test_utils {
    use super::*;

    pub struct TestContext {
        pub program: Program<'static, RaydiumCpi>,
        pub payer: Keypair,
        pub pool: Pubkey,
        pub pool_authority: Pubkey,
        pub tick_array: Pubkey,
    }

    impl TestContext {
        pub async fn new() -> Self {
            let program = Program::new();
            let payer = Keypair::new();
            let pool = Keypair::new();

            let (pool_authority, _) = Pubkey::find_program_address(
                &[
                    seeds::POOL_AUTHORITY,
                    pool.pubkey().as_ref(),
                ],
                &program.id(),
            );

            let start_tick_index = 0;
            let (tick_array, _) = Pubkey::find_program_address(
                &[
                    seeds::TICK_ARRAY,
                    pool.pubkey().as_ref(),
                    start_tick_index.to_le_bytes().as_ref(),
                ],
                &program.id(),
            );

            Self {
                program,
                payer,
                pool: pool.pubkey(),
                pool_authority,
                tick_array,
            }
        }

        pub async fn initialize_tick_array(
            &self,
            start_tick_index: i32,
        ) -> Result<()> {
            self.program.request()
                .accounts(InitializeTickArray {
                    owner: self.payer.pubkey(),
                    pool: self.pool.clone(),
                    tick_array: self.tick_array,
                    system_program: System::id(),
                })
                .args(InitializeTickArrayArgs {
                    start_tick_index,
                })
                .signer(&self.payer)
                .send()
                .await
        }

        pub async fn update_tick(
            &self,
            tick_index: i32,
            liquidity_net: i128,
            liquidity_gross: u128,
            fee_growth_outside_0_x64: u128,
            fee_growth_outside_1_x64: u128,
        ) -> Result<()> {
            self.program.request()
                .accounts(UpdateTick {
                    owner: self.payer.pubkey(),
                    pool: self.pool.clone(),
                    tick_array: self.tick_array,
                })
                .args(UpdateTickArgs {
                    tick_index,
                    liquidity_net,
                    liquidity_gross,
                    fee_growth_outside_0_x64,
                    fee_growth_outside_1_x64,
                })
                .signer(&self.payer)
                .send()
                .await
        }

        pub async fn cross_tick(
            &self,
            tick_index: i32,
        ) -> Result<()> {
            self.program.request()
                .accounts(CrossTick {
                    owner: self.payer.pubkey(),
                    pool: self.pool.clone(),
                    tick_array: self.tick_array,
                })
                .args(CrossTickArgs {
                    tick_index,
                })
                .signer(&self.payer)
                .send()
                .await
        }
    }
}

#[tokio::test]
async fn test_initialize_tick_array() {
    let ctx = TestContext::new().await;

    // Test successful initialization
    let result = ctx.initialize_tick_array(0).await;
    assert!(result.is_ok(), "Tick array initialization should succeed");

    // Test invalid tick index (not multiple of spacing)
    let result = ctx.initialize_tick_array(3).await;
    assert!(result.is_err(), "Should fail with invalid tick index");

    // Test tick index out of bounds
    let result = ctx.initialize_tick_array(MAX_TICK + 1).await;
    assert!(result.is_err(), "Should fail with tick index out of bounds");
}

#[tokio::test]
async fn test_update_tick() {
    let ctx = TestContext::new().await;

    // Initialize tick array
    ctx.initialize_tick_array(0).await.unwrap();

    // Test successful update
    let result = ctx.update_tick(
        0,
        100,
        100,
        0,
        0,
    ).await;
    assert!(result.is_ok(), "Tick update should succeed");

    // Test update with invalid tick index
    let result = ctx.update_tick(
        TICK_ARRAY_SIZE as i32,
        100,
        100,
        0,
        0,
    ).await;
    assert!(result.is_err(), "Should fail with tick index out of bounds");

    // Test update with max values
    let result = ctx.update_tick(
        0,
        i128::MAX,
        u128::MAX,
        u128::MAX,
        u128::MAX,
    ).await;
    assert!(result.is_ok(), "Should handle max values");
}

#[tokio::test]
async fn test_cross_tick() {
    let ctx = TestContext::new().await;

    // Initialize tick array
    ctx.initialize_tick_array(0).await.unwrap();

    // Update tick with some liquidity
    ctx.update_tick(
        0,
        100,
        100,
        0,
        0,
    ).await.unwrap();

    // Test successful tick crossing
    let result = ctx.cross_tick(0).await;
    assert!(result.is_ok(), "Tick crossing should succeed");

    // Test crossing invalid tick
    let result = ctx.cross_tick(TICK_ARRAY_SIZE as i32).await;
    assert!(result.is_err(), "Should fail with tick index out of bounds");

    // Test crossing tick multiple times
    let result = ctx.cross_tick(0).await;
    assert!(result.is_ok(), "Should handle multiple crossings");
}

#[tokio::test]
async fn test_tick_array_boundaries() {
    let ctx = TestContext::new().await;

    // Test at min tick
    let result = ctx.initialize_tick_array(MIN_TICK).await;
    assert!(result.is_ok(), "Should initialize at min tick");

    // Test at max tick
    let result = ctx.initialize_tick_array(MAX_TICK).await;
    assert!(result.is_ok(), "Should initialize at max tick");

    // Test updating tick at boundaries
    ctx.initialize_tick_array(0).await.unwrap();
    
    let result = ctx.update_tick(
        0,
        100,
        100,
        0,
        0,
    ).await;
    assert!(result.is_ok(), "Should update first tick");

    let result = ctx.update_tick(
        TICK_ARRAY_SIZE as i32 - 1,
        100,
        100,
        0,
        0,
    ).await;
    assert!(result.is_ok(), "Should update last tick");
}
