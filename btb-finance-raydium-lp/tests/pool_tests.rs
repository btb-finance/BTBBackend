use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use raydium_cpi::{state::*, utils::*, instructions::pool::*};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};

pub mod test_utils {
    use super::*;

    pub struct TestContext {
        pub program: Program<'static, RaydiumCpi>,
        pub payer: Keypair,
        pub token_mint_0: Keypair,
        pub token_mint_1: Keypair,
        pub token_vault_0: Keypair,
        pub token_vault_1: Keypair,
        pub pool: Pubkey,
        pub pool_authority: Pubkey,
    }

    impl TestContext {
        pub async fn new() -> Self {
            let program = Program::new();
            let payer = Keypair::new();
            let token_mint_0 = Keypair::new();
            let token_mint_1 = Keypair::new();
            let token_vault_0 = Keypair::new();
            let token_vault_1 = Keypair::new();

            // Ensure token_mint_0 < token_mint_1
            let (token_mint_0, token_mint_1) = if token_mint_0.pubkey() < token_mint_1.pubkey() {
                (token_mint_0, token_mint_1)
            } else {
                (token_mint_1, token_mint_0)
            };

            let pool_config = PoolConfig {
                fee_rate: 3000, // 0.3%
                protocol_fee_rate: 10, // 10%
                tick_spacing: 1,
            };

            let (pool, _) = Pubkey::find_program_address(
                &[
                    seeds::POOL,
                    token_mint_0.pubkey().as_ref(),
                    token_mint_1.pubkey().as_ref(),
                    pool_config.fee_rate.to_le_bytes().as_ref(),
                ],
                &program.id(),
            );

            let (pool_authority, _) = Pubkey::find_program_address(
                &[
                    seeds::POOL_AUTHORITY,
                    pool.as_ref(),
                ],
                &program.id(),
            );

            Self {
                program,
                payer,
                token_mint_0,
                token_mint_1,
                token_vault_0,
                token_vault_1,
                pool,
                pool_authority,
            }
        }

        pub async fn create_pool(
            &self,
            fee_rate: u32,
            protocol_fee_rate: u8,
            tick_spacing: u16,
        ) -> Result<()> {
            self.program.request()
                .accounts(CreatePool {
                    owner: self.payer.pubkey(),
                    pool: self.pool,
                    token_mint_0: self.token_mint_0.pubkey(),
                    token_mint_1: self.token_mint_1.pubkey(),
                    token_vault_0: self.token_vault_0.pubkey(),
                    token_vault_1: self.token_vault_1.pubkey(),
                    pool_authority: self.pool_authority,
                    token_program: token::ID,
                    system_program: System::id(),
                })
                .args(PoolConfig {
                    fee_rate,
                    protocol_fee_rate,
                    tick_spacing,
                })
                .signer(&self.payer)
                .send()
                .await
        }

        pub async fn update_pool(
            &self,
            fee_rate: Option<u32>,
            protocol_fee_rate: Option<u8>,
        ) -> Result<()> {
            self.program.request()
                .accounts(UpdatePool {
                    owner: self.payer.pubkey(),
                    pool: self.pool,
                })
                .args((fee_rate, protocol_fee_rate))
                .signer(&self.payer)
                .send()
                .await
        }

        pub async fn collect_protocol_fees(
            &self,
            recipient_token_account_0: Pubkey,
            recipient_token_account_1: Pubkey,
        ) -> Result<()> {
            self.program.request()
                .accounts(CollectProtocolFees {
                    owner: self.payer.pubkey(),
                    pool: self.pool,
                    token_vault_0: self.token_vault_0.pubkey(),
                    token_vault_1: self.token_vault_1.pubkey(),
                    recipient_token_account_0,
                    recipient_token_account_1,
                    pool_authority: self.pool_authority,
                    token_program: token::ID,
                })
                .signer(&self.payer)
                .send()
                .await
        }
    }
}

#[tokio::test]
async fn test_create_pool() {
    let ctx = TestContext::new().await;

    // Test successful pool creation
    let result = ctx.create_pool(3000, 10, 1).await;
    assert!(result.is_ok(), "Pool creation should succeed");

    // Test invalid fee rate
    let result = ctx.create_pool(MAX_FEE_RATE + 1, 10, 1).await;
    assert!(result.is_err(), "Should fail with invalid fee rate");

    // Test invalid protocol fee rate
    let result = ctx.create_pool(3000, MAX_PROTOCOL_FEE_RATE + 1, 1).await;
    assert!(result.is_err(), "Should fail with invalid protocol fee rate");

    // Test invalid tick spacing
    let result = ctx.create_pool(3000, 10, MAX_TICK_SPACING + 1).await;
    assert!(result.is_err(), "Should fail with invalid tick spacing");
}

#[tokio::test]
async fn test_update_pool() {
    let ctx = TestContext::new().await;

    // Create pool first
    ctx.create_pool(3000, 10, 1).await.unwrap();

    // Test successful fee rate update
    let result = ctx.update_pool(Some(2000), None).await;
    assert!(result.is_ok(), "Fee rate update should succeed");

    // Test successful protocol fee rate update
    let result = ctx.update_pool(None, Some(15)).await;
    assert!(result.is_ok(), "Protocol fee rate update should succeed");

    // Test invalid fee rate
    let result = ctx.update_pool(Some(MAX_FEE_RATE + 1), None).await;
    assert!(result.is_err(), "Should fail with invalid fee rate");

    // Test invalid protocol fee rate
    let result = ctx.update_pool(None, Some(MAX_PROTOCOL_FEE_RATE + 1)).await;
    assert!(result.is_err(), "Should fail with invalid protocol fee rate");
}

#[tokio::test]
async fn test_collect_protocol_fees() {
    let ctx = TestContext::new().await;

    // Create pool first
    ctx.create_pool(3000, 10, 1).await.unwrap();

    // Create recipient token accounts
    let recipient_token_account_0 = Keypair::new();
    let recipient_token_account_1 = Keypair::new();

    // Test successful fee collection
    let result = ctx.collect_protocol_fees(
        recipient_token_account_0.pubkey(),
        recipient_token_account_1.pubkey(),
    ).await;
    assert!(result.is_ok(), "Fee collection should succeed");

    // Test fee collection with zero fees
    let result = ctx.collect_protocol_fees(
        recipient_token_account_0.pubkey(),
        recipient_token_account_1.pubkey(),
    ).await;
    assert!(result.is_ok(), "Fee collection with zero fees should succeed");
}

#[tokio::test]
async fn test_pool_boundaries() {
    let ctx = TestContext::new().await;

    // Test min values
    let result = ctx.create_pool(0, MIN_PROTOCOL_FEE_RATE, 1).await;
    assert!(result.is_ok(), "Should succeed with min values");

    // Test max values
    let result = ctx.create_pool(MAX_FEE_RATE, MAX_PROTOCOL_FEE_RATE, MAX_TICK_SPACING).await;
    assert!(result.is_ok(), "Should succeed with max values");

    // Test mixed boundaries
    let result = ctx.create_pool(MAX_FEE_RATE, MIN_PROTOCOL_FEE_RATE, MAX_TICK_SPACING).await;
    assert!(result.is_ok(), "Should succeed with mixed boundaries");
}
