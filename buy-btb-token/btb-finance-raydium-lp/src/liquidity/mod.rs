use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

mod add;
mod remove;
mod position;

pub use add::*;
pub use remove::*;
pub use position::*;

/// Configuration for adding liquidity to BTB.finance pools
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AddLiquidityConfig {
    /// Amount of token A to add
    pub token_amount_a: u64,
    /// Amount of token B to add
    pub token_amount_b: u64,
    /// Lower price bound for position
    pub price_lower: u64,
    /// Upper price bound for position
    pub price_upper: u64,
    /// Maximum allowed slippage (e.g., 100 = 1%)
    pub slippage_tolerance: u16,
}

/// Configuration for removing liquidity from BTB.finance pools
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RemoveLiquidityConfig {
    /// Percentage of liquidity to remove (e.g., 10000 = 100%)
    pub percentage: u16,
    /// Minimum amount of token A to receive
    pub min_token_a: u64,
    /// Minimum amount of token B to receive
    pub min_token_b: u64,
}

/// Account validation for liquidity operations
#[derive(Accounts)]
pub struct LiquidityAccounts<'info> {
    /// The user adding/removing liquidity
    #[account(mut)]
    pub user: Signer<'info>,

    /// The pool to interact with
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// User's token A account
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,

    /// User's token B account
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    /// Pool's token A vault
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,

    /// Pool's token B vault
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,

    /// The token program
    pub token_program: Program<'info, Token>,
}

/// Pool state containing core pool information
#[account]
pub struct Pool {
    /// Pool authority
    pub authority: Pubkey,
    /// Token A mint
    pub token_a_mint: Pubkey,
    /// Token B mint
    pub token_b_mint: Pubkey,
    /// Current pool fee rate (e.g., 30 = 0.3%)
    pub fee_rate: u16,
    /// Total liquidity in the pool
    pub liquidity: u128,
    /// Current sqrt price
    pub sqrt_price_x64: u128,
    /// Current tick index
    pub tick_current: i32,
    /// Tick spacing
    pub tick_spacing: u16,
    /// Fee growth for token A
    pub fee_growth_global_a: u128,
    /// Fee growth for token B
    pub fee_growth_global_b: u128,
}
