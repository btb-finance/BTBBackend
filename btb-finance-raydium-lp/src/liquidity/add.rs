use anchor_lang::prelude::*;
use anchor_spl::token;

use crate::{
    errors::ErrorCode,
    utils::{calc_amount_delta, price_to_tick, tick_to_price},
};

use super::{AddLiquidityConfig, LiquidityAccounts, Pool};

/// Configuration for adding liquidity
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AddLiquidityConfig {
    pub price_lower: u128,
    pub price_upper: u128,
    pub token_amount_a: u64,
    pub token_amount_b: u64,
    pub slippage_tolerance: u16,  // In basis points (1 bp = 0.01%)
}

/// Add liquidity to a BTB.finance pool
pub fn add_liquidity(
    ctx: Context<LiquidityAccounts>,
    config: AddLiquidityConfig,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    
    // Validate price range
    let tick_lower = price_to_tick(config.price_lower)?;
    let tick_upper = price_to_tick(config.price_upper)?;
    require!(tick_lower < tick_upper, ErrorCode::InvalidPriceRange);
    require!(
        tick_lower % pool.tick_spacing as i32 == 0 
        && tick_upper % pool.tick_spacing as i32 == 0,
        ErrorCode::InvalidTickSpacing
    );

    // Calculate liquidity from amounts
    let sqrt_price = pool.sqrt_price_x64;
    let liquidity = calc_amount_delta(
        sqrt_price,
        tick_to_price(tick_lower)?,
        tick_to_price(tick_upper)?,
        config.token_amount_a,
        config.token_amount_b,
    )?;

    // Calculate actual token amounts needed with slippage protection
    let (min_amount_a, min_amount_b) = calculate_min_amounts(
        liquidity,
        sqrt_price,
        tick_lower,
        tick_upper,
        config.slippage_tolerance,
    )?;

    // Verify amounts are within slippage tolerance
    require!(
        config.token_amount_a >= min_amount_a && 
        config.token_amount_b >= min_amount_b,
        ErrorCode::SlippageExceeded
    );

    // Calculate actual token amounts needed based on current tick
    let (amount_a, amount_b) = if pool.tick_current < tick_lower {
        // Only token A
        (config.token_amount_a, 0)
    } else if pool.tick_current < tick_upper {
        // Both tokens
        (config.token_amount_a, config.token_amount_b)
    } else {
        // Only token B
        (0, config.token_amount_b)
    };

    // Transfer tokens to pool
    if amount_a > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.pool_token_a.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_a,
        )?;
    }

    if amount_b > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.pool_token_b.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount_b,
        )?;
    }

    // Update pool state
    pool.liquidity = pool.liquidity.checked_add(liquidity)
        .ok_or(ErrorCode::LiquidityOverflow)?;

    // Emit liquidity added event
    emit!(LiquidityAddedEvent {
        pool: ctx.accounts.pool.key(),
        user: ctx.accounts.user.key(),
        liquidity,
        amount_a,
        amount_b,
        tick_lower,
        tick_upper,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Calculate minimum token amounts based on slippage tolerance
fn calculate_min_amounts(
    liquidity: u128,
    sqrt_price: u128,
    tick_lower: i32,
    tick_upper: i32,
    slippage_tolerance: u16,
) -> Result<(u64, u64)> {
    let sqrt_price_lower = tick_math::get_sqrt_price_at_tick(tick_lower)?;
    let sqrt_price_upper = tick_math::get_sqrt_price_at_tick(tick_upper)?;
    
    // Calculate ideal amounts
    let amount_0 = calculate_amount0(liquidity, sqrt_price_lower, sqrt_price_upper)?;
    let amount_1 = calculate_amount1(liquidity, sqrt_price_lower, sqrt_price_upper)?;
    
    // Apply slippage tolerance
    let slippage_factor = (10000u128 - slippage_tolerance as u128) as f64 / 10000f64;
    let min_amount_0 = (amount_0 as f64 * slippage_factor) as u64;
    let min_amount_1 = (amount_1 as f64 * slippage_factor) as u64;
    
    Ok((min_amount_0, min_amount_1))
}

/// Event emitted when liquidity is added
#[event]
pub struct LiquidityAddedEvent {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub liquidity: u128,
    pub amount_a: u64,
    pub amount_b: u64,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub timestamp: i64,
}
