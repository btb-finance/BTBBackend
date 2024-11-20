use anchor_lang::prelude::*;

/// State account for a Raydium liquidity pool
#[account]
#[derive(Default)]
pub struct LiquidityPool {
    /// Bump seed for PDA derivation
    pub bump: u8,
    /// Mint address of token A
    pub token_a_mint: Pubkey,
    /// Mint address of token B
    pub token_b_mint: Pubkey,
    /// Vault holding token A reserves
    pub token_a_vault: Pubkey,
    /// Vault holding token B reserves
    pub token_b_vault: Pubkey,
    /// Minimum tick spacing for the pool
    pub tick_spacing: u16,
    /// Fee rate in basis points (1 = 0.01%)
    pub fee_rate: u64,
    /// Current pool liquidity
    pub liquidity: u128,
    /// Current sqrt price * 2^64
    pub sqrt_price_x64: u128,
    /// Current tick index
    pub current_tick: i32,
    /// Index for price observation
    pub observation_index: u16,
    /// Minimum time between observations
    pub observation_update_duration: u16,
}

/// State account for a liquidity position
#[account]
#[derive(Default)]
pub struct LiquidityPosition {
    /// Pool this position belongs to
    pub pool: Pubkey,
    /// Lower tick index defining position range
    pub tick_lower: i32,
    /// Upper tick index defining position range
    pub tick_upper: i32,
    /// Amount of liquidity owned at this position
    pub liquidity: u128,
    /// Accumulated fees in token A
    pub fee_growth_inside_a: u128,
    /// Accumulated fees in token B
    pub fee_growth_inside_b: u128,
    /// Unclaimed token A fees
    pub tokens_owed_a: u64,
    /// Unclaimed token B fees
    pub tokens_owed_b: u64,
}

/// State account for tick data storage
#[account]
#[derive(Default)]
pub struct TickArray {
    /// Starting tick index for this array
    pub start_tick: i32,
    /// Individual tick data
    pub ticks: Vec<TickData>,
}

/// Data for a single tick in the pool
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TickData {
    /// Whether this tick has been initialized
    pub initialized: bool,
    /// Net liquidity change when tick is crossed
    pub liquidity_net: i128,
    /// Total liquidity at this tick
    pub liquidity_gross: u128,
    /// Fee growth outside tick in token A
    pub fee_growth_outside_a: u128,
    /// Fee growth outside tick in token B
    pub fee_growth_outside_b: u128,
}
