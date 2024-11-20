use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use raydium_cpi::{state::*, utils::*, instructions::router::*};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

mod test_utils;
use test_utils::*;

#[tokio::test]
async fn test_single_swap() {
    let ctx = TestContext::new().await;

    // Create position for liquidity
    let position = ctx.create_position(-100, 100, 1_000_000)
        .await
        .expect("Failed to create position");

    // Perform swap
    let amount_in = 10_000;
    let min_amount_out = 9_900; // 1% slippage
    let sqrt_price_limit = MIN_SQRT_PRICE_X64;

    let config = SwapStepConfig {
        amount_in,
        min_amount_out,
        sqrt_price_limit_x64: sqrt_price_limit,
        zero_for_one: true,
    };

    let amount_out = ctx.swap_step(config)
        .await
        .expect("Failed to perform swap");

    assert!(amount_out >= min_amount_out, "Swap output below minimum");

    // Verify token balances
    let token_a_balance = ctx.get_token_balance(ctx.token_a_account)
        .await
        .expect("Failed to get token A balance");
    let token_b_balance = ctx.get_token_balance(ctx.token_b_account)
        .await
        .expect("Failed to get token B balance");

    assert_eq!(token_a_balance, amount_in, "Incorrect token A balance");
    assert_eq!(token_b_balance, amount_out, "Incorrect token B balance");
}

#[tokio::test]
async fn test_router_swap() {
    let ctx = TestContext::new().await;

    // Create positions in both pools
    let position1 = ctx.create_position(-100, 100, 1_000_000)
        .await
        .expect("Failed to create position in pool 1");

    let position2 = ctx.create_position(-100, 100, 1_000_000)
        .await
        .expect("Failed to create position in pool 2");

    // Create intermediate token
    let mint_c = Keypair::new();
    let token_c_account = Keypair::new();

    // Perform router swap
    let amount_in = 10_000;
    let min_amount_out = 9_800; // 2% slippage for two hops

    let config = RouterConfig {
        amount_in,
        min_amount_out,
        route: vec![ctx.pool1, ctx.pool2],
    };

    let amount_out = ctx.router_swap(config)
        .await
        .expect("Failed to perform router swap");

    assert!(amount_out >= min_amount_out, "Router swap output below minimum");

    // Verify final token balances
    let token_a_balance = ctx.get_token_balance(ctx.token_a_account)
        .await
        .expect("Failed to get token A balance");
    let token_b_balance = ctx.get_token_balance(ctx.token_b_account)
        .await
        .expect("Failed to get token B balance");

    assert_eq!(token_a_balance, amount_in, "Incorrect token A balance");
    assert_eq!(token_b_balance, amount_out, "Incorrect token B balance");
}

#[tokio::test]
async fn test_swap_validation() {
    let ctx = TestContext::new().await;

    // Test invalid price limit
    let config = SwapStepConfig {
        amount_in: 10_000,
        min_amount_out: 9_900,
        sqrt_price_limit_x64: MIN_SQRT_PRICE_X64 - 1, // Below minimum
        zero_for_one: true,
    };

    let result = ctx.swap_step(config).await;
    assert!(result.is_err(), "Should fail with invalid price limit");

    // Test excessive slippage
    let config = SwapStepConfig {
        amount_in: 10_000,
        min_amount_out: 10_001, // Impossible (requesting more out than in)
        sqrt_price_limit_x64: MIN_SQRT_PRICE_X64,
        zero_for_one: true,
    };

    let result = ctx.swap_step(config).await;
    assert!(result.is_err(), "Should fail with excessive slippage");

    // Test zero amount
    let config = SwapStepConfig {
        amount_in: 0,
        min_amount_out: 0,
        sqrt_price_limit_x64: MIN_SQRT_PRICE_X64,
        zero_for_one: true,
    };

    let result = ctx.swap_step(config).await;
    assert!(result.is_err(), "Should fail with zero amount");
}

#[tokio::test]
async fn test_router_validation() {
    let ctx = TestContext::new().await;

    // Test empty route
    let config = RouterConfig {
        amount_in: 10_000,
        min_amount_out: 9_800,
        route: vec![],
    };

    let result = ctx.router_swap(config).await;
    assert!(result.is_err(), "Should fail with empty route");

    // Test invalid pool in route
    let config = RouterConfig {
        amount_in: 10_000,
        min_amount_out: 9_800,
        route: vec![Pubkey::new_unique()], // Random invalid pool
    };

    let result = ctx.router_swap(config).await;
    assert!(result.is_err(), "Should fail with invalid pool");

    // Test incompatible tokens in route
    let config = RouterConfig {
        amount_in: 10_000,
        min_amount_out: 9_800,
        route: vec![ctx.pool1, ctx.pool3], // Pools with incompatible tokens
    };

    let result = ctx.router_swap(config).await;
    assert!(result.is_err(), "Should fail with incompatible tokens");
}

// Helper implementation for test context
impl TestContext {
    pub async fn swap_step(
        &self,
        config: SwapStepConfig,
    ) -> Result<u64> {
        self.program.request()
            .accounts(SwapStep {
                pool: self.pool,
                token_account_in: self.token_a_account,
                token_account_out: self.token_b_account,
                token_vault_0: self.token_vault_0,
                token_vault_1: self.token_vault_1,
                tick_array: self.tick_array,
                pool_authority: self.pool_authority,
                token_program: token::ID,
            })
            .args(config)
            .signer(&self.payer)
            .send()
            .await
            .map(|()| 0) // TODO: Return actual amount out
    }

    pub async fn router_swap(
        &self,
        config: RouterConfig,
    ) -> Result<u64> {
        self.program.request()
            .accounts(RouterSwap {
                authority: self.payer.pubkey(),
                token_account_in: self.token_a_account,
                token_account_out: self.token_b_account,
                token_program: token::ID,
            })
            .args(config)
            .signer(&self.payer)
            .send()
            .await
            .map(|()| 0) // TODO: Return actual amount out
    }
}
