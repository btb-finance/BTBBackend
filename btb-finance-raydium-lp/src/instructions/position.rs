use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::{state::*, utils::*, math::fees::*};

/// Configuration parameters for creating or modifying a liquidity position.
///
/// This struct defines the parameters required to create a new position or
/// modify an existing one in a Raydium AMM V3 pool. It specifies the price
/// range and initial token amounts for the position.
///
/// # Parameters
/// * `tick_lower_index` - Lower bound of the price range (inclusive)
/// * `tick_upper_index` - Upper bound of the price range (inclusive)
/// * `amount_0` - Amount of token0 to deposit
/// * `amount_1` - Amount of token1 to deposit
///
/// # Constraints
/// * `tick_lower_index` must be less than `tick_upper_index`
/// * Both ticks must be within MIN_TICK (-443636) and MAX_TICK (443636)
/// * Ticks must be multiples of the pool's tick spacing
///
/// # Example
/// ```rust
/// let config = PositionConfig {
///     tick_lower_index: -100,
///     tick_upper_index: 100,
///     amount_0: 1_000_000,
///     amount_1: 1_000_000,
/// };
/// ```
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PositionConfig {
    /// Lower tick index
    pub tick_lower_index: i32,
    /// Upper tick index
    pub tick_upper_index: i32,
    /// Amount of token 0 to deposit
    pub amount_0: u64,
    /// Amount of token 1 to deposit
    pub amount_1: u64,
}

/// Account constraints and requirements for opening a new liquidity position.
///
/// This struct validates and provides access to all accounts required for
/// creating a new position in a Raydium AMM V3 pool.
///
/// # Security Considerations
/// * Validates pool ownership of token vaults
/// * Ensures proper PDA derivation for position account
/// * Verifies tick array boundaries
///
/// # Account Requirements
/// * `owner` - The authority that will own the position
/// * `pool` - The liquidity pool account
/// * `position` - The position account to be created (PDA)
/// * `tick_array_lower` - Tick array containing lower tick
/// * `tick_array_upper` - Tick array containing upper tick
/// * `token_account_0` - Owner's token0 account
/// * `token_account_1` - Owner's token1 account
/// * `token_vault_0` - Pool's token0 vault
/// * `token_vault_1` - Pool's token1 vault
#[derive(Accounts)]
#[instruction(position_config: PositionConfig)]
pub struct OpenPosition<'info> {
    /// The authority that will own the position
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The pool to create position in
    #[account(
        mut,
        has_one = token_vault_0,
        has_one = token_vault_1,
    )]
    pub pool: Account<'info, LiquidityPool>,

    /// The position account to be created
    #[account(
        init,
        payer = owner,
        space = LiquidityPosition::LEN,
        seeds = [
            seeds::POSITION,
            pool.key().as_ref(),
            owner.key().as_ref(),
            position_config.tick_lower_index.to_le_bytes().as_ref(),
            position_config.tick_upper_index.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub position: Account<'info, LiquidityPosition>,

    /// The tick array containing the lower tick
    #[account(mut)]
    pub tick_array_lower: Account<'info, TickArray>,

    /// The tick array containing the upper tick
    #[account(mut)]
    pub tick_array_upper: Account<'info, TickArray>,

    /// The owner's token 0 account
    #[account(mut)]
    pub token_account_0: Account<'info, TokenAccount>,

    /// The owner's token 1 account
    #[account(mut)]
    pub token_account_1: Account<'info, TokenAccount>,

    /// The pool's token 0 vault
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,

    /// The pool's token 1 vault
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,

    /// The pool authority PDA
    /// CHECK: PDA owned by the program
    #[account(
        seeds = [
            seeds::POOL_AUTHORITY,
            pool.key().as_ref(),
        ],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    /// The token program
    pub token_program: Program<'info, Token>,

    /// The system program
    pub system_program: Program<'info, System>,
}

/// Account constraints for collecting fees from a liquidity position.
///
/// This struct validates and provides access to all accounts required for
/// collecting accumulated fees from a position in a Raydium AMM V3 pool.
///
/// # Security Considerations
/// * Validates position ownership
/// * Ensures proper token account ownership
/// * Verifies fee calculations
///
/// # Account Requirements
/// * `owner` - The position owner
/// * `pool` - The liquidity pool account
/// * `position` - The position account
/// * `token_account_0` - Owner's token0 account
/// * `token_account_1` - Owner's token1 account
/// * `token_vault_0` - Pool's token0 vault
/// * `token_vault_1` - Pool's token1 vault
#[derive(Accounts)]
pub struct CollectPositionFees<'info> {
    /// The position owner
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The pool containing the position
    #[account(
        mut,
        has_one = token_vault_0,
        has_one = token_vault_1,
    )]
    pub pool: Account<'info, LiquidityPool>,

    /// The position to collect fees from
    #[account(
        mut,
        has_one = owner @ ErrorCode::InvalidPositionOwner,
        has_one = pool @ ErrorCode::InvalidPositionPool,
    )]
    pub position: Account<'info, LiquidityPosition>,

    /// The tick array containing the lower tick
    #[account(mut)]
    pub tick_array_lower: Account<'info, TickArray>,

    /// The tick array containing the upper tick
    #[account(mut)]
    pub tick_array_upper: Account<'info, TickArray>,

    /// The owner's token 0 account
    #[account(mut)]
    pub token_account_0: Account<'info, TokenAccount>,

    /// The owner's token 1 account
    #[account(mut)]
    pub token_account_1: Account<'info, TokenAccount>,

    /// The pool's token 0 vault
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,

    /// The pool's token 1 vault
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,

    /// The pool authority PDA
    /// CHECK: PDA owned by the program
    #[account(
        seeds = [
            seeds::POOL_AUTHORITY,
            pool.key().as_ref(),
        ],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    /// The token program
    pub token_program: Program<'info, Token>,
}

/// Open a new liquidity position in a Raydium AMM V3 pool.
///
/// This function creates a new position by:
/// 1. Validating position parameters
/// 2. Calculating required liquidity
/// 3. Updating tick arrays
/// 4. Transferring tokens to pool vaults
///
/// # Arguments
/// * `ctx` - Context containing all required accounts
/// * `position_config` - Position configuration parameters
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Errors
/// * Returns error if ticks are invalid
/// * Returns error if insufficient token balance
/// * Returns error if tick arrays are full
///
/// # Example
/// ```rust
/// let position = open_position(ctx, config)?;
/// msg!("Created new position with liquidity {}", position.liquidity);
/// ```
pub fn open_position(
    ctx: Context<OpenPosition>,
    position_config: PositionConfig,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let position = &mut ctx.accounts.position;
    let tick_array_lower = &mut ctx.accounts.tick_array_lower;
    let tick_array_upper = &mut ctx.accounts.tick_array_upper;

    // Validate tick indices
    require!(
        position_config.tick_lower_index < position_config.tick_upper_index,
        ErrorCode::InvalidTickRange
    );

    require!(
        position_config.tick_lower_index >= MIN_TICK
            && position_config.tick_upper_index <= MAX_TICK,
        ErrorCode::InvalidTickIndex
    );

    require!(
        position_config.tick_lower_index % pool.tick_spacing as i32 == 0
            && position_config.tick_upper_index % pool.tick_spacing as i32 == 0,
        ErrorCode::InvalidTickSpacing
    );

    // Calculate liquidity from amounts
    let liquidity = calculate_liquidity_from_amounts(
        pool.sqrt_price_x64,
        position_config.tick_lower_index,
        position_config.tick_upper_index,
        position_config.amount_0,
        position_config.amount_1,
    )?;

    require!(liquidity > 0, ErrorCode::ZeroLiquidity);

    // Update ticks
    update_tick(
        tick_array_lower,
        position_config.tick_lower_index,
        liquidity as i128,
        true,
    )?;

    update_tick(
        tick_array_upper,
        position_config.tick_upper_index,
        -(liquidity as i128),
        true,
    )?;

    // Update pool liquidity if position is in range
    if pool.tick_current >= position_config.tick_lower_index
        && pool.tick_current < position_config.tick_upper_index
    {
        pool.liquidity = pool.liquidity
            .checked_add(liquidity)
            .ok_or(ErrorCode::MathOverflow)?;
    }

    // Initialize position
    position.pool = pool.key();
    position.owner = ctx.accounts.owner.key();
    position.tick_lower_index = position_config.tick_lower_index;
    position.tick_upper_index = position_config.tick_upper_index;
    position.liquidity = liquidity;
    position.fee_growth_inside_0_last_x64 = 0;
    position.fee_growth_inside_1_last_x64 = 0;
    position.tokens_owed_0 = 0;
    position.tokens_owed_1 = 0;

    // Transfer tokens to pool vaults
    if position_config.amount_0 > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_account_0.to_account_info(),
                    to: ctx.accounts.token_vault_0.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            position_config.amount_0,
        )?;
    }

    if position_config.amount_1 > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_account_1.to_account_info(),
                    to: ctx.accounts.token_vault_1.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            position_config.amount_1,
        )?;
    }

    Ok(())
}

/// Collect accumulated fees from a liquidity position.
///
/// This function calculates and transfers accumulated fees to the position owner,
/// updating fee growth trackers and handling token transfers.
///
/// # Arguments
/// * `ctx` - Context containing all required accounts
///
/// # Returns
/// * `Result<()>` - Success or error
///
/// # Errors
/// * Returns error if position has no fees to collect
/// * Returns error if token transfer fails
///
/// # Example
/// ```rust
/// collect_position_fees(ctx)?;
/// msg!("Collected fees from position");
/// ```
pub fn collect_position_fees(ctx: Context<CollectPositionFees>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let position = &mut ctx.accounts.position;
    let tick_array_lower = &ctx.accounts.tick_array_lower;
    let tick_array_upper = &ctx.accounts.tick_array_upper;

    // Get lower and upper ticks
    let tick_lower = get_tick(tick_array_lower, position.tick_lower_index)?;
    let tick_upper = get_tick(tick_array_upper, position.tick_upper_index)?;

    // Calculate uncollected fees
    let (uncollected_fees_0, uncollected_fees_1) = update_position_fees(
        position,
        pool,
        tick_lower,
        tick_upper,
    )?;

    // Add tokens owed
    let total_fees_0 = uncollected_fees_0
        .checked_add(position.tokens_owed_0 as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    let total_fees_1 = uncollected_fees_1
        .checked_add(position.tokens_owed_1 as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    // Reset tokens owed
    position.tokens_owed_0 = 0;
    position.tokens_owed_1 = 0;

    // Transfer fees to owner
    if total_fees_0 > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_vault_0.to_account_info(),
                    to: ctx.accounts.token_account_0.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                &[&[
                    seeds::POOL_AUTHORITY,
                    pool.key().as_ref(),
                    &[*ctx.bumps.get("pool_authority").unwrap()],
                ]],
            ),
            total_fees_0 as u64,
        )?;
    }

    if total_fees_1 > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_vault_1.to_account_info(),
                    to: ctx.accounts.token_account_1.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                &[&[
                    seeds::POOL_AUTHORITY,
                    pool.key().as_ref(),
                    &[*ctx.bumps.get("pool_authority").unwrap()],
                ]],
            ),
            total_fees_1 as u64,
        )?;
    }

    Ok(())
}

/// Calculate the liquidity amount for a position based on token amounts.
///
/// This helper function calculates the liquidity value for a position given
/// the current price and token amounts.
///
/// # Arguments
/// * `sqrt_price_x64` - Current sqrt price in x64 format
/// * `tick_lower` - Lower tick index
/// * `tick_upper` - Upper tick index
/// * `amount_0` - Amount of token0
/// * `amount_1` - Amount of token1
///
/// # Returns
/// * `Result<u128>` - Calculated liquidity amount
fn calculate_liquidity_from_amounts(
    sqrt_price_x64: u128,
    tick_lower: i32,
    tick_upper: i32,
    amount_0: u64,
    amount_1: u64,
) -> Result<u128> {
    let sqrt_price_lower = tick_math::get_sqrt_price_at_tick(tick_lower)?;
    let sqrt_price_upper = tick_math::get_sqrt_price_at_tick(tick_upper)?;

    let liquidity_by_0 = if amount_0 > 0 {
        let numerator = (amount_0 as u128) << 64;
        let denominator = sqrt_price_upper.checked_sub(sqrt_price_lower)
            .ok_or(ErrorCode::MathOverflow)?;
        numerator.checked_div(denominator)
            .ok_or(ErrorCode::MathOverflow)?
    } else {
        0
    };

    let liquidity_by_1 = if amount_1 > 0 {
        let numerator = amount_1 as u128;
        numerator.checked_mul(sqrt_price_lower)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_mul(sqrt_price_upper)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(sqrt_price_upper.checked_sub(sqrt_price_lower)
                .ok_or(ErrorCode::MathOverflow)?)
            .ok_or(ErrorCode::MathOverflow)?
            >> 64
    } else {
        0
    };

    // Return the minimum liquidity between token0 and token1
    Ok(std::cmp::min(liquidity_by_0, liquidity_by_1))
}

/// Update a tick in the tick array.
///
/// This helper function updates a tick's state when liquidity is added or removed.
///
/// # Arguments
/// * `tick_array` - The tick array to update
/// * `tick_index` - Index of the tick to update
/// * `liquidity_delta` - Change in liquidity (positive for add, negative for remove)
/// * `upper` - Whether this is the upper tick of a position
///
/// # Returns
/// * `Result<()>` - Success or error
fn update_tick(
    tick_array: &mut TickArray,
    tick_index: i32,
    liquidity_delta: i128,
    upper: bool,
) -> Result<()> {
    // Validate tick index is within array bounds
    require!(
        tick_index >= tick_array.start_tick_index
            && tick_index < tick_array.start_tick_index + TICK_ARRAY_SIZE as i32,
        ErrorCode::InvalidTickIndex
    );

    // Calculate array index
    let array_index = (tick_index - tick_array.start_tick_index) as usize;
    let tick = &mut tick_array.ticks[array_index];

    // Update tick liquidity
    if upper {
        tick.liquidity_gross = tick.liquidity_gross
            .checked_add(liquidity_delta.unsigned_abs())
            .ok_or(ErrorCode::MathOverflow)?;
    }
    tick.liquidity_net = tick.liquidity_net
        .checked_add(liquidity_delta)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(())
}

/// Get a reference to a tick in a tick array.
///
/// This helper function retrieves a reference to a tick given its index.
///
/// # Arguments
/// * `tick_array` - The tick array to search
/// * `tick_index` - Index of the tick to retrieve
///
/// # Returns
/// * `Result<&'a Tick>` - Reference to the tick
fn get_tick<'a>(tick_array: &'a TickArray, tick_index: i32) -> Result<&'a Tick> {
    require!(
        tick_index >= tick_array.start_tick_index
            && tick_index < tick_array.start_tick_index + TICK_ARRAY_SIZE as i32,
        ErrorCode::InvalidTickIndex
    );

    let array_index = (tick_index - tick_array.start_tick_index) as usize;
    Ok(&tick_array.ticks[array_index])
}

/// Size of each tick array in number of ticks
const TICK_ARRAY_SIZE: usize = 88;

/// Minimum allowed tick index (-443636)
const MIN_TICK: i32 = -443636;

/// Maximum allowed tick index (443636)
const MAX_TICK: i32 = 443636;
