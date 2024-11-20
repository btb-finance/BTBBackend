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
        pub mint_c: Pubkey,
        pub token_a_account: Pubkey,
        pub token_b_account: Pubkey,
        pub token_c_account: Pubkey,
        pub pool_ab: Pubkey,
        pub pool_bc: Pubkey,
        pub pool_ab_authority: Pubkey,
        pub pool_bc_authority: Pubkey,
        pub token_a_vault: Pubkey,
        pub token_b_vault_1: Pubkey,
        pub token_b_vault_2: Pubkey,
        pub token_c_vault: Pubkey,
        pub tick_array_ab: Pubkey,
        pub tick_array_bc: Pubkey,
    }

    impl TestContext {
        pub async fn new() -> Self {
            let program = Program::new();
            let payer = Keypair::new();

            // Create token mints
            let mint_a = Keypair::new();
            let mint_b = Keypair::new();
            let mint_c = Keypair::new();
            
            // Create token accounts
            let token_a_account = Keypair::new();
            let token_b_account = Keypair::new();
            let token_c_account = Keypair::new();

            // Create pools and vaults
            let pool_ab = Keypair::new();
            let pool_bc = Keypair::new();

            let (pool_ab_authority, _) = Pubkey::find_program_address(
                &[
                    seeds::POOL_AUTHORITY,
                    pool_ab.pubkey().as_ref(),
                ],
                &program.id(),
            );

            let (pool_bc_authority, _) = Pubkey::find_program_address(
                &[
                    seeds::POOL_AUTHORITY,
                    pool_bc.pubkey().as_ref(),
                ],
                &program.id(),
            );

            let token_a_vault = Keypair::new();
            let token_b_vault_1 = Keypair::new();
            let token_b_vault_2 = Keypair::new();
            let token_c_vault = Keypair::new();

            // Create tick arrays
            let tick_array_ab = Keypair::new();
            let tick_array_bc = Keypair::new();

            Self {
                program,
                payer,
                mint_a: mint_a.pubkey(),
                mint_b: mint_b.pubkey(),
                mint_c: mint_c.pubkey(),
                token_a_account: token_a_account.pubkey(),
                token_b_account: token_b_account.pubkey(),
                token_c_account: token_c_account.pubkey(),
                pool_ab: pool_ab.pubkey(),
                pool_bc: pool_bc.pubkey(),
                pool_ab_authority,
                pool_bc_authority,
                token_a_vault: token_a_vault.pubkey(),
                token_b_vault_1: token_b_vault_1.pubkey(),
                token_b_vault_2: token_b_vault_2.pubkey(),
                token_c_vault: token_c_vault.pubkey(),
                tick_array_ab: tick_array_ab.pubkey(),
                tick_array_bc: tick_array_bc.pubkey(),
            }
        }

        pub async fn single_swap(
            &self,
            amount_in: u64,
            amount_out_minimum: u64,
            sqrt_price_limit: u128,
            zero_for_one: bool,
        ) -> Result<()> {
            self.program.request()
                .accounts(Swap {
                    owner: self.payer.pubkey(),
                    pool: self.pool_ab,
                    source_token_account: self.token_a_account,
                    destination_token_account: self.token_b_account,
                    token_a_vault: self.token_a_vault,
                    token_b_vault: self.token_b_vault_1,
                    pool_authority: self.pool_ab_authority,
                    tick_array: self.tick_array_ab,
                    token_program: token::ID,
                })
                .args(SwapArgs {
                    amount_in,
                    amount_out_minimum,
                    sqrt_price_limit,
                    zero_for_one,
                })
                .signer(&self.payer)
                .send()
                .await
        }

        pub async fn router_swap(
            &self,
            amount_in: u64,
            amount_out_minimum: u64,
            sqrt_price_limit_1: u128,
            sqrt_price_limit_2: u128,
        ) -> Result<()> {
            self.program.request()
                .accounts(SwapRouterContext {
                    owner: self.payer.pubkey(),
                    pool_1: self.pool_ab,
                    pool_2: self.pool_bc,
                    source_token_account: self.token_a_account,
                    intermediate_token_account: self.token_b_account,
                    destination_token_account: self.token_c_account,
                    pool_1_token_a_vault: self.token_a_vault,
                    pool_1_token_b_vault: self.token_b_vault_1,
                    pool_2_token_a_vault: self.token_b_vault_2,
                    pool_2_token_b_vault: self.token_c_vault,
                    pool_1_authority: self.pool_ab_authority,
                    pool_2_authority: self.pool_bc_authority,
                    pool_1_tick_array: self.tick_array_ab,
                    pool_2_tick_array: self.tick_array_bc,
                    token_program: token::ID,
                })
                .args(SwapRouterArgs {
                    amount_in,
                    amount_out_minimum,
                    sqrt_price_limit_1,
                    sqrt_price_limit_2,
                })
                .signer(&self.payer)
                .send()
                .await
        }
    }
}

#[tokio::test]
async fn test_single_swap() {
    let ctx = TestContext::new().await;

    // Test successful swap
    let result = ctx.single_swap(
        1_000_000,
        900_000,
        u128::MAX,
        true,
    ).await;
    assert!(result.is_ok(), "Swap should succeed");

    // Test slippage protection
    let result = ctx.single_swap(
        1_000_000,
        1_100_000, // Expecting more output than possible
        u128::MAX,
        true,
    ).await;
    assert!(result.is_err(), "Swap should fail due to slippage");

    // Test zero amount
    let result = ctx.single_swap(
        0,
        0,
        u128::MAX,
        true,
    ).await;
    assert!(result.is_err(), "Swap should fail with zero amount");

    // Test invalid price limit
    let result = ctx.single_swap(
        1_000_000,
        900_000,
        0,
        true,
    ).await;
    assert!(result.is_err(), "Swap should fail with invalid price limit");
}

#[tokio::test]
async fn test_router_swap() {
    let ctx = TestContext::new().await;

    // Test successful router swap
    let result = ctx.router_swap(
        1_000_000,
        800_000,
        u128::MAX,
        u128::MAX,
    ).await;
    assert!(result.is_ok(), "Router swap should succeed");

    // Test slippage protection
    let result = ctx.router_swap(
        1_000_000,
        1_100_000, // Expecting more output than possible
        u128::MAX,
        u128::MAX,
    ).await;
    assert!(result.is_err(), "Router swap should fail due to slippage");

    // Test zero amount
    let result = ctx.router_swap(
        0,
        0,
        u128::MAX,
        u128::MAX,
    ).await;
    assert!(result.is_err(), "Router swap should fail with zero amount");

    // Test invalid price limits
    let result = ctx.router_swap(
        1_000_000,
        800_000,
        0,
        u128::MAX,
    ).await;
    assert!(result.is_err(), "Router swap should fail with invalid first price limit");

    let result = ctx.router_swap(
        1_000_000,
        800_000,
        u128::MAX,
        0,
    ).await;
    assert!(result.is_err(), "Router swap should fail with invalid second price limit");
}
