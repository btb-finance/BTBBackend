use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

/// Account validation for fee collection
#[derive(Accounts)]
pub struct CollectFeesAccounts<'info> {
    /// The position owner collecting fees
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The pool containing the position
    #[account(mut)]
    pub pool: Account<'info, crate::liquidity::Pool>,

    /// The position to collect fees from
    #[account(
        mut,
        has_one = owner @ ErrorCode::InvalidPositionOwner,
        has_one = pool @ ErrorCode::InvalidPositionPool,
    )]
    pub position: Account<'info, Position>,

    /// Owner's token A account
    #[account(mut)]
    pub token_account_a: Account<'info, TokenAccount>,

    /// Owner's token B account
    #[account(mut)]
    pub token_account_b: Account<'info, TokenAccount>,

    /// Pool's token A vault
    #[account(mut)]
    pub pool_token_a: Account<'info, TokenAccount>,

    /// Pool's token B vault
    #[account(mut)]
    pub pool_token_b: Account<'info, TokenAccount>,

    /// The token program
    pub token_program: Program<'info, Token>,
}

/// Position state for tracking fees
#[account]
pub struct Position {
    /// Position owner
    pub owner: Pubkey,
    /// Pool this position belongs to
    pub pool: Pubkey,
    /// Lower tick index
    pub tick_lower: i32,
    /// Upper tick index
    pub tick_upper: i32,
    /// Position liquidity
    pub liquidity: u128,
    /// Fee growth of token A inside position
    pub fee_growth_inside_a: u128,
    /// Fee growth of token B inside position
    pub fee_growth_inside_b: u128,
    /// Tokens owed to position from fees (token A)
    pub tokens_owed_a: u64,
    /// Tokens owed to position from fees (token B)
    pub tokens_owed_b: u64,
}

/// Collect fees from a position
pub fn collect_fees(ctx: Context<CollectFeesAccounts>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let position = &mut ctx.accounts.position;

    // Calculate fees earned since last collection
    let fee_delta_a = pool.fee_growth_global_a
        .checked_sub(position.fee_growth_inside_a)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(position.liquidity)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(u128::pow(2, 64))
        .ok_or(ErrorCode::MathOverflow)? as u64;

    let fee_delta_b = pool.fee_growth_global_b
        .checked_sub(position.fee_growth_inside_b)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(position.liquidity)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(u128::pow(2, 64))
        .ok_or(ErrorCode::MathOverflow)? as u64;

    // Add to tokens owed
    position.tokens_owed_a = position.tokens_owed_a
        .checked_add(fee_delta_a)
        .ok_or(ErrorCode::MathOverflow)?;
    position.tokens_owed_b = position.tokens_owed_b
        .checked_add(fee_delta_b)
        .ok_or(ErrorCode::MathOverflow)?;

    // Transfer owed tokens
    if position.tokens_owed_a > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.pool_token_a.to_account_info(),
                    to: ctx.accounts.token_account_a.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[&[
                    b"pool_authority",
                    pool.key().as_ref(),
                    &[pool.authority.bump],
                ]],
            ),
            position.tokens_owed_a,
        )?;
        position.tokens_owed_a = 0;
    }

    if position.tokens_owed_b > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.pool_token_b.to_account_info(),
                    to: ctx.accounts.token_account_b.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[&[
                    b"pool_authority",
                    pool.key().as_ref(),
                    &[pool.authority.bump],
                ]],
            ),
            position.tokens_owed_b,
        )?;
        position.tokens_owed_b = 0;
    }

    // Update fee growth trackers
    position.fee_growth_inside_a = pool.fee_growth_global_a;
    position.fee_growth_inside_b = pool.fee_growth_global_b;

    Ok(())
}

// Example usage:
/*
    use btb_finance_clmm::fees::collect_fees;

    // Collect fees from a position
    collect_fees(ctx)?;

    msg!("Successfully collected fees from position");
*/
