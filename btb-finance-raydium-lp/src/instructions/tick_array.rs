use anchor_lang::prelude::*;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
pub struct InitializeTickArray<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool: AccountInfo<'info>,

    #[account(
        init,
        payer = owner,
        space = TickArray::LEN,
        seeds = [
            seeds::TICK_ARRAY,
            pool.key().as_ref(),
            start_tick_index.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub tick_array: Account<'info, TickArray>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeTickArrayArgs {
    pub start_tick_index: i32,
}

pub fn initialize_tick_array(
    ctx: Context<InitializeTickArray>,
    args: InitializeTickArrayArgs,
) -> Result<()> {
    let tick_array = &mut ctx.accounts.tick_array;
    let pool = &ctx.accounts.pool;

    // Validate tick index
    require!(
        is_valid_tick_index(args.start_tick_index),
        ErrorCode::InvalidTickIndex
    );

    // Initialize tick array
    tick_array.pool = pool.key();
    tick_array.start_tick_index = args.start_tick_index;
    tick_array.ticks = vec![Tick::default(); TICK_ARRAY_SIZE].try_into().unwrap();

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateTick<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            seeds::TICK_ARRAY,
            pool.key().as_ref(),
            tick_array.start_tick_index.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub tick_array: Account<'info, TickArray>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateTickArgs {
    pub tick_index: i32,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside_0_x64: u128,
    pub fee_growth_outside_1_x64: u128,
}

pub fn update_tick(ctx: Context<UpdateTick>, args: UpdateTickArgs) -> Result<()> {
    let tick_array = &mut ctx.accounts.tick_array;

    // Validate tick index is within array bounds
    let tick_index = args.tick_index;
    require!(
        tick_index >= tick_array.start_tick_index
            && tick_index < tick_array.start_tick_index + TICK_ARRAY_SIZE as i32,
        ErrorCode::InvalidTickIndex
    );

    // Calculate array index
    let array_index = (tick_index - tick_array.start_tick_index) as usize;

    // Update tick
    let tick = &mut tick_array.ticks[array_index];
    tick.liquidity_net = args.liquidity_net;
    tick.liquidity_gross = args.liquidity_gross;
    tick.fee_growth_outside_0_x64 = args.fee_growth_outside_0_x64;
    tick.fee_growth_outside_1_x64 = args.fee_growth_outside_1_x64;

    Ok(())
}

#[derive(Accounts)]
pub struct CrossTick<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        seeds = [
            seeds::TICK_ARRAY,
            pool.key().as_ref(),
            tick_array.start_tick_index.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub tick_array: Account<'info, TickArray>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CrossTickArgs {
    pub tick_index: i32,
}

pub fn cross_tick(ctx: Context<CrossTick>, args: CrossTickArgs) -> Result<()> {
    let tick_array = &mut ctx.accounts.tick_array;
    let pool = &mut ctx.accounts.pool;

    // Validate tick index is within array bounds
    let tick_index = args.tick_index;
    require!(
        tick_index >= tick_array.start_tick_index
            && tick_index < tick_array.start_tick_index + TICK_ARRAY_SIZE as i32,
        ErrorCode::InvalidTickIndex
    );

    // Calculate array index
    let array_index = (tick_index - tick_array.start_tick_index) as usize;
    let tick = &mut tick_array.ticks[array_index];

    // Update pool liquidity
    pool.liquidity = if pool.tick_current >= tick_index {
        pool.liquidity
            .checked_sub(tick.liquidity_net as u128)
            .ok_or(ErrorCode::MathOverflow)?
    } else {
        pool.liquidity
            .checked_add(tick.liquidity_net as u128)
            .ok_or(ErrorCode::MathOverflow)?
    };

    // Update fee growth
    tick.fee_growth_outside_0_x64 = pool.fee_growth_global_0_x64.wrapping_sub(
        tick.fee_growth_outside_0_x64
    );
    tick.fee_growth_outside_1_x64 = pool.fee_growth_global_1_x64.wrapping_sub(
        tick.fee_growth_outside_1_x64
    );

    Ok(())
}

// Helper functions
fn is_valid_tick_index(tick_index: i32) -> bool {
    tick_index % TICK_SPACING == 0
        && tick_index >= MIN_TICK
        && tick_index <= MAX_TICK
}

// Constants
pub const TICK_ARRAY_SIZE: usize = 88;
pub const TICK_SPACING: i32 = 1;
pub const MIN_TICK: i32 = -443636;
pub const MAX_TICK: i32 = 443636;
