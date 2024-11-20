use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use raydium_clmm_sdk::{
    states::{Position, Pool, PositionInfo},
    errors::ErrorCode as RaydiumError,
};

pub mod errors;
pub mod instructions;
pub mod liquidity;
pub mod math;
pub mod optimizer;
pub mod states;
pub mod utils;

declare_id!("BTBFiNZy5K4WS9TFYxT8vpRcHBJq3CDf3DTh8NaGDDYP");

#[program]
pub mod btb_finance_clmm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        tick_spacing: u16,
        initial_price: u128,
    ) -> Result<()> {
        // Input validation
        require!(tick_spacing > 0, RaydiumError::InvalidParameters);
        require!(initial_price > 0, RaydiumError::InvalidParameters);
        require!(
            ctx.accounts.pool.owner == ctx.accounts.payer.key(),
            RaydiumError::InvalidAuthority
        );
        
        instructions::pools::initialize_pool(ctx, tick_spacing, initial_price)
    }

    pub fn create_position(
        ctx: Context<CreatePosition>,
        tick_lower_index: i32,
        tick_upper_index: i32,
        liquidity: u128,
        amount_0_max: u64,
        amount_1_max: u64,
    ) -> Result<()> {
        // Input validation
        require!(liquidity > 0, RaydiumError::InvalidLiquidity);
        require!(tick_lower_index < tick_upper_index, RaydiumError::InvalidTickRange);
        require!(
            amount_0_max > 0 || amount_1_max > 0,
            RaydiumError::InvalidParameters
        );
        
        // Validate token accounts
        require!(
            ctx.accounts.token_account_0.mint == ctx.accounts.token_vault_0.mint,
            RaydiumError::TokenMintMismatch
        );
        require!(
            ctx.accounts.token_account_1.mint == ctx.accounts.token_vault_1.mint,
            RaydiumError::TokenMintMismatch
        );
        
        instructions::positions::create_position(
            ctx,
            tick_lower_index,
            tick_upper_index,
            liquidity,
            amount_0_max,
            amount_1_max,
        )
    }

    pub fn increase_liquidity(
        ctx: Context<ModifyPosition>,
        liquidity: u128,
        amount_0_max: u64,
        amount_1_max: u64,
    ) -> Result<()> {
        // Reentrancy check
        require!(!ctx.accounts.pool.is_locked(), RaydiumError::Reentrancy);
        
        // Input validation
        require!(liquidity > 0, RaydiumError::InvalidLiquidity);
        require!(
            amount_0_max > 0 || amount_1_max > 0,
            RaydiumError::InvalidParameters
        );
        
        // Owner validation
        require!(
            ctx.accounts.position.owner == ctx.accounts.owner.key(),
            RaydiumError::InvalidOwner
        );
        
        instructions::positions::increase_liquidity(ctx, liquidity, amount_0_max, amount_1_max)
    }

    pub fn decrease_liquidity(
        ctx: Context<ModifyPosition>,
        liquidity: u128,
        amount_0_min: u64,
        amount_1_min: u64,
    ) -> Result<()> {
        instructions::positions::decrease_liquidity(ctx, liquidity, amount_0_min, amount_1_min)
    }

    pub fn collect_fees(
        ctx: Context<CollectFees>,
        amount_0_requested: u64,
        amount_1_requested: u64,
    ) -> Result<()> {
        instructions::positions::collect_fees(ctx, amount_0_requested, amount_1_requested)
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        instructions::positions::close_position(ctx)
    }
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    /// CHECK: Validated in CPI
    #[account(mut)]
    pub pool: AccountInfo<'info>,
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,
    #[account(mut)]
    pub observation_state: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: Validated in CPI
    pub raydium_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreatePosition<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,
    #[account(mut)]
    pub position_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_account_0: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_account_1: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,
    #[account(mut)]
    pub tick_array_lower: AccountInfo<'info>,
    #[account(mut)]
    pub tick_array_upper: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: Validated in CPI
    pub raydium_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ModifyPosition<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub position_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_account_0: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_account_1: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,
    #[account(mut)]
    pub tick_array_lower: AccountInfo<'info>,
    #[account(mut)]
    pub tick_array_upper: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Validated in CPI
    pub raydium_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub position_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_account_0: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_account_1: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Validated in CPI
    pub raydium_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub position_nft_mint: Account<'info, TokenAccount>,
    #[account(mut)]
    pub position_nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Validated in CPI
    pub raydium_program: AccountInfo<'info>,
}
