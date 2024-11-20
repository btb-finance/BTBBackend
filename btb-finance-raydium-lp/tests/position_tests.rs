use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use raydium_cpi::{state::*, utils::*};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

pub mod test_utils {
    use super::*;

    pub struct TestContext {
        pub program: Program<'static, RaydiumCpi>,
        pub payer: Keypair,
        pub mint_a: Pubkey,
        pub mint_b: Pubkey,
        pub token_a_account: Pubkey,
        pub token_b_account: Pubkey,
        pub pool: Pubkey,
        pub pool_authority: Pubkey,
        pub token_a_vault: Pubkey,
        pub token_b_vault: Pubkey,
        pub tick_array_lower: Pubkey,
        pub tick_array_upper: Pubkey,
    }

    impl TestContext {
        pub async fn new() -> Self {
            let program = Program::new();
            let payer = Keypair::new();

            // Create token mints
            let mint_a = Keypair::new();
            let mint_b = Keypair::new();
            
            // Create token accounts
            let token_a_account = Keypair::new();
            let token_b_account = Keypair::new();

            // Create pool and vaults
            let pool = Keypair::new();
            let (pool_authority, _) = Pubkey::find_program_address(
                &[
                    seeds::POOL_AUTHORITY,
                    pool.pubkey().as_ref(),
                ],
                &program.id(),
            );

            let token_a_vault = Keypair::new();
            let token_b_vault = Keypair::new();

            let (tick_array_lower, _) = Pubkey::find_program_address(
                &[
                    seeds::TICK_ARRAY,
                    pool.pubkey().as_ref(),
                    &[0u8],
                ],
                &program.id(),
            );

            let (tick_array_upper, _) = Pubkey::find_program_address(
                &[
                    seeds::TICK_ARRAY,
                    pool.pubkey().as_ref(),
                    &[1u8],
                ],
                &program.id(),
            );

            Self {
                program,
                payer,
                mint_a: mint_a.pubkey(),
                mint_b: mint_b.pubkey(),
                token_a_account: token_a_account.pubkey(),
                token_b_account: token_b_account.pubkey(),
                pool: pool.pubkey(),
                pool_authority,
                token_a_vault: token_a_vault.pubkey(),
                token_b_vault: token_b_vault.pubkey(),
                tick_array_lower,
                tick_array_upper,
            }
        }

        pub async fn create_position(
            &self,
            tick_lower: i32,
            tick_upper: i32,
            liquidity_amount: u128,
        ) -> Result<Pubkey> {
            let position = Keypair::new();
            let (position_pda, bump) = Pubkey::find_program_address(
                &[
                    seeds::POSITION,
                    self.pool.as_ref(),
                    self.payer.pubkey().as_ref(),
                ],
                &self.program.id(),
            );

            self.program.request()
                .accounts(OpenPosition {
                    owner: self.payer.pubkey(),
                    pool: self.pool,
                    position: position_pda,
                    token_a_account: self.token_a_account,
                    token_b_account: self.token_b_account,
                    token_a_vault: self.token_a_vault,
                    token_b_vault: self.token_b_vault,
                    pool_authority: self.pool_authority,
                    token_program: token::ID,
                    system_program: system_program::ID,
                })
                .args(OpenPositionArgs {
                    tick_lower,
                    tick_upper,
                    liquidity_amount,
                })
                .signer(&self.payer)
                .send()
                .await
                .map(|_| position_pda)
        }

        pub async fn remove_liquidity(
            &self,
            position: Pubkey,
            liquidity_amount: u128,
        ) -> Result<()> {
            self.program.request()
                .accounts(RemoveLiquidity {
                    owner: self.payer.pubkey(),
                    pool: self.pool,
                    position,
                    token_a_account: self.token_a_account,
                    token_b_account: self.token_b_account,
                    token_a_vault: self.token_a_vault,
                    token_b_vault: self.token_b_vault,
                    pool_authority: self.pool_authority,
                    tick_array_lower: self.tick_array_lower,
                    tick_array_upper: self.tick_array_upper,
                    token_program: token::ID,
                })
                .args(RemoveLiquidityArgs {
                    liquidity_amount,
                })
                .signer(&self.payer)
                .send()
                .await
        }

        pub async fn close_position(
            &self,
            position: Pubkey,
        ) -> Result<()> {
            self.program.request()
                .accounts(ClosePosition {
                    owner: self.payer.pubkey(),
                    pool: self.pool,
                    position,
                    token_a_account: self.token_a_account,
                    token_b_account: self.token_b_account,
                    token_a_vault: self.token_a_vault,
                    token_b_vault: self.token_b_vault,
                    pool_authority: self.pool_authority,
                    token_program: token::ID,
                    system_program: system_program::ID,
                })
                .signer(&self.payer)
                .send()
                .await
        }

        pub async fn simulate_swaps(
            &self,
            position: Pubkey,
            amount_0: u64,
            amount_1: u64,
        ) -> Result<()> {
            // Simulate swaps by directly updating pool state
            // Note: In a real implementation, perform actual swaps
            let mut pool_account = self.program
                .account::<LiquidityPool>(self.pool)
                .await?;

            pool_account.fee_growth_global_0_x64 = pool_account.fee_growth_global_0_x64
                .checked_add(amount_0 as u128)
                .ok_or(ErrorCode::MathOverflow)?;

            pool_account.fee_growth_global_1_x64 = pool_account.fee_growth_global_1_x64
                .checked_add(amount_1 as u128)
                .ok_or(ErrorCode::MathOverflow)?;

            Ok(())
        }

        pub async fn collect_fees(
            &self,
            position: Pubkey,
        ) -> Result<(u64, u64)> {
            self.program.request()
                .accounts(CollectPositionFees {
                    owner: self.payer.pubkey(),
                    pool: self.pool,
                    position,
                    tick_array_lower: self.tick_array_lower,
                    tick_array_upper: self.tick_array_upper,
                    token_account_0: self.token_a_account,
                    token_account_1: self.token_b_account,
                    token_vault_0: self.token_a_vault,
                    token_vault_1: self.token_b_vault,
                    pool_authority: self.pool_authority,
                    token_program: token::ID,
                })
                .signer(&self.payer)
                .send()
                .await?;

            Ok((
                self.get_token_balance(self.token_a_account).await?,
                self.get_token_balance(self.token_b_account).await?,
            ))
        }

        pub async fn get_token_balance(&self, token_account: Pubkey) -> Result<u64> {
            let account = self.program
                .account::<TokenAccount>(token_account)
                .await?;
            Ok(account.amount)
        }
    }
}

#[tokio::test]
async fn test_position_lifecycle() {
    let ctx = TestContext::new().await;

    // Create position
    let position = ctx.create_position(-100, 100, 1_000_000)
        .await
        .expect("Failed to create position");

    // Remove some liquidity
    ctx.remove_liquidity(position, 500_000)
        .await
        .expect("Failed to remove liquidity");

    // Remove remaining liquidity
    ctx.remove_liquidity(position, 500_000)
        .await
        .expect("Failed to remove remaining liquidity");

    // Close position
    ctx.close_position(position)
        .await
        .expect("Failed to close position");
}

#[tokio::test]
async fn test_position_validation() {
    let ctx = TestContext::new().await;

    // Test invalid tick range
    let result = ctx.create_position(100, 0, 1_000_000).await;
    assert!(result.is_err(), "Should fail with invalid tick range");

    // Test zero liquidity
    let result = ctx.create_position(-100, 100, 0).await;
    assert!(result.is_err(), "Should fail with zero liquidity");

    // Create valid position
    let position = ctx.create_position(-100, 100, 1_000_000)
        .await
        .expect("Failed to create position");

    // Test removing too much liquidity
    let result = ctx.remove_liquidity(position, 2_000_000).await;
    assert!(result.is_err(), "Should fail with insufficient liquidity");

    // Test closing position with remaining liquidity
    let result = ctx.close_position(position).await;
    assert!(result.is_err(), "Should fail to close position with remaining liquidity");
}

#[tokio::test]
async fn test_fee_collection() {
    let ctx = TestContext::new().await;

    // Create position
    let position = ctx.create_position(-100, 100, 1_000_000)
        .await
        .expect("Failed to create position");

    // Simulate some swaps to generate fees
    // Note: In a real test, we would perform actual swaps
    ctx.simulate_swaps(position, 1000, 1000)
        .await
        .expect("Failed to simulate swaps");

    // Collect fees
    let (fees_0, fees_1) = ctx.collect_fees(position)
        .await
        .expect("Failed to collect fees");

    assert!(fees_0 > 0, "Should have collected token 0 fees");
    assert!(fees_1 > 0, "Should have collected token 1 fees");

    // Verify fees were transferred
    let token_a_balance = ctx.get_token_balance(ctx.token_a_account)
        .await
        .expect("Failed to get token A balance");
    let token_b_balance = ctx.get_token_balance(ctx.token_b_account)
        .await
        .expect("Failed to get token B balance");

    assert_eq!(token_a_balance, fees_0, "Token A balance should match collected fees");
    assert_eq!(token_b_balance, fees_1, "Token B balance should match collected fees");
}

#[tokio::test]
async fn test_position_edge_cases() {
    let ctx = TestContext::new().await;

    // Test minimum tick spacing
    let result = ctx.create_position(-10, 10, 1_000_000).await;
    assert!(result.is_err(), "Should fail with invalid tick spacing");

    // Test maximum tick range
    let result = ctx.create_position(MIN_TICK - 1, 100, 1_000_000).await;
    assert!(result.is_err(), "Should fail with tick below minimum");

    let result = ctx.create_position(-100, MAX_TICK + 1, 1_000_000).await;
    assert!(result.is_err(), "Should fail with tick above maximum");

    // Test position at price range boundaries
    let position = ctx.create_position(MIN_TICK, MAX_TICK, 1_000_000)
        .await
        .expect("Failed to create full range position");

    // Test collecting fees with no accumulated fees
    let (fees_0, fees_1) = ctx.collect_fees(position)
        .await
        .expect("Failed to collect fees");

    assert_eq!(fees_0, 0, "Should have no token 0 fees");
    assert_eq!(fees_1, 0, "Should have no token 1 fees");

    // Test removing liquidity in multiple steps
    ctx.remove_liquidity(position, 400_000)
        .await
        .expect("Failed to remove first portion");

    ctx.remove_liquidity(position, 400_000)
        .await
        .expect("Failed to remove second portion");

    ctx.remove_liquidity(position, 200_000)
        .await
        .expect("Failed to remove final portion");

    // Verify position can be closed after all liquidity removed
    ctx.close_position(position)
        .await
        .expect("Failed to close position");
}

// Constants
const MIN_TICK: i32 = -443636;
const MAX_TICK: i32 = 443636;
