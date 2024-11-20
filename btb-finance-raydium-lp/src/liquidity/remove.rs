use anchor_lang::prelude::*;
use anchor_spl::token;

use crate::{
    errors::ErrorCode,
    utils::calc_withdraw_amounts,
};

use super::{RemoveLiquidityConfig, LiquidityAccounts, Pool};

/// Remove liquidity from a BTB.finance pool
pub fn remove_liquidity(
    ctx: Context<LiquidityAccounts>,
    config: RemoveLiquidityConfig,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    
    // Validate percentage
    require!(config.percentage <= 10000, ErrorCode::InvalidPercentage);

    // Calculate liquidity to remove
    let liquidity_to_remove = pool.liquidity
        .checked_mul(config.percentage as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::MathOverflow)?;

    // Calculate token amounts to withdraw
    let (amount_a, amount_b) = calc_withdraw_amounts(
        pool.sqrt_price_x64,
        pool.tick_current,
        liquidity_to_remove,
    )?;

    // Verify minimum amounts
    require!(amount_a >= config.min_token_a, ErrorCode::SlippageExceeded);
    require!(amount_b >= config.min_token_b, ErrorCode::SlippageExceeded);

    // Transfer tokens from pool
    if amount_a > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.pool_token_a.to_account_info(),
                    to: ctx.accounts.user_token_a.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[&[
                    b"pool_authority",
                    pool.key().as_ref(),
                    &[pool.authority.bump],
                ]],
            ),
            amount_a,
        )?;
    }

    if amount_b > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.pool_token_b.to_account_info(),
                    to: ctx.accounts.user_token_b.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[&[
                    b"pool_authority",
                    pool.key().as_ref(),
                    &[pool.authority.bump],
                ]],
            ),
            amount_b,
        )?;
    }

    // Update pool state
    pool.liquidity = pool.liquidity.checked_sub(liquidity_to_remove)
        .ok_or(ErrorCode::InsufficientLiquidity)?;

    Ok(())
}

// Example usage:
/*
    use btb_finance_clmm::liquidity::{remove_liquidity, RemoveLiquidityConfig};

    // Remove 50% of liquidity from a BTB.finance pool
    let config = RemoveLiquidityConfig {
        percentage: 5000,         // 50%
        min_token_a: 450_000,    // Minimum 0.45 USDC
        min_token_b: 450_000_000, // Minimum 0.45 SOL
    };

    remove_liquidity(ctx, config)?;
*/
