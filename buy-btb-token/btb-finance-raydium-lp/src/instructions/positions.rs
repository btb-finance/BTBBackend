use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use raydium_clmm_sdk::states::{Position, Pool};

use crate::{
    CreatePosition,
    ModifyPosition,
    CollectFees,
    ClosePosition,
    errors::ErrorCode,
};

pub fn create_position(
    ctx: Context<CreatePosition>,
    tick_lower_index: i32,
    tick_upper_index: i32,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
) -> Result<()> {
    // Validate parameters
    require!(
        tick_lower_index < tick_upper_index,
        ErrorCode::InvalidTickRange
    );

    require!(liquidity > 0, ErrorCode::InvalidLiquidity);

    // Create CPI context
    let cpi_accounts = raydium_clmm_sdk::accounts::CreatePosition {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        position_nft_mint: ctx.accounts.position_nft_mint.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        token_account_0: ctx.accounts.token_account_0.to_account_info(),
        token_account_1: ctx.accounts.token_account_1.to_account_info(),
        token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
        token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
        tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
        tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.raydium_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute CPI call
    raydium_clmm_sdk::cpi::create_position(
        cpi_ctx,
        tick_lower_index,
        tick_upper_index,
        liquidity,
        amount_0_max,
        amount_1_max,
    ).map_err(|_| ErrorCode::CpiCallFailed)?;

    // Emit position creation event
    emit!(PositionCreatedEvent {
        pool: ctx.accounts.pool.key(),
        position: ctx.accounts.position.key(),
        owner: ctx.accounts.owner.key(),
        tick_lower_index,
        tick_upper_index,
        liquidity,
        amount_0_max,
        amount_1_max,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn increase_liquidity(
    ctx: Context<ModifyPosition>,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
) -> Result<()> {
    // Validate parameters
    require!(liquidity > 0, ErrorCode::InvalidLiquidity);

    // Create CPI context
    let cpi_accounts = raydium_clmm_sdk::accounts::ModifyPosition {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        token_account_0: ctx.accounts.token_account_0.to_account_info(),
        token_account_1: ctx.accounts.token_account_1.to_account_info(),
        token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
        token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
        tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
        tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.raydium_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute CPI call
    raydium_clmm_sdk::cpi::increase_liquidity(
        cpi_ctx,
        liquidity,
        amount_0_max,
        amount_1_max,
    ).map_err(|_| ErrorCode::CpiCallFailed)?;

    // Emit liquidity increase event
    emit!(LiquidityIncreasedEvent {
        pool: ctx.accounts.pool.key(),
        position: ctx.accounts.position.key(),
        owner: ctx.accounts.owner.key(),
        liquidity,
        amount_0_max,
        amount_1_max,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn decrease_liquidity(
    ctx: Context<ModifyPosition>,
    liquidity: u128,
    amount_0_min: u64,
    amount_1_min: u64,
) -> Result<()> {
    // Validate parameters
    require!(liquidity > 0, ErrorCode::InvalidLiquidity);

    // Create CPI context
    let cpi_accounts = raydium_clmm_sdk::accounts::ModifyPosition {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        token_account_0: ctx.accounts.token_account_0.to_account_info(),
        token_account_1: ctx.accounts.token_account_1.to_account_info(),
        token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
        token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
        tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
        tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.raydium_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute CPI call
    raydium_clmm_sdk::cpi::decrease_liquidity(
        cpi_ctx,
        liquidity,
        amount_0_min,
        amount_1_min,
    ).map_err(|_| ErrorCode::CpiCallFailed)?;

    // Emit liquidity decrease event
    emit!(LiquidityDecreasedEvent {
        pool: ctx.accounts.pool.key(),
        position: ctx.accounts.position.key(),
        owner: ctx.accounts.owner.key(),
        liquidity,
        amount_0_min,
        amount_1_min,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn collect_fees(
    ctx: Context<CollectFees>,
    amount_0_requested: u64,
    amount_1_requested: u64,
) -> Result<()> {
    // Create CPI context
    let cpi_accounts = raydium_clmm_sdk::accounts::CollectFees {
        pool: ctx.accounts.pool.to_account_info(),
        position: ctx.accounts.position.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        token_account_0: ctx.accounts.token_account_0.to_account_info(),
        token_account_1: ctx.accounts.token_account_1.to_account_info(),
        token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
        token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.raydium_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute CPI call
    raydium_clmm_sdk::cpi::collect_fees(
        cpi_ctx,
        amount_0_requested,
        amount_1_requested,
    ).map_err(|_| ErrorCode::CpiCallFailed)?;

    // Emit fee collection event
    emit!(FeesCollectedEvent {
        pool: ctx.accounts.pool.key(),
        position: ctx.accounts.position.key(),
        owner: ctx.accounts.owner.key(),
        amount_0_collected: amount_0_requested,
        amount_1_collected: amount_1_requested,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
    // Create CPI context
    let cpi_accounts = raydium_clmm_sdk::accounts::ClosePosition {
        position: ctx.accounts.position.to_account_info(),
        position_nft_mint: ctx.accounts.position_nft_mint.to_account_info(),
        position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
        owner: ctx.accounts.owner.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.raydium_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute CPI call
    raydium_clmm_sdk::cpi::close_position(cpi_ctx)
        .map_err(|_| ErrorCode::CpiCallFailed)?;

    // Emit position closure event
    emit!(PositionClosedEvent {
        position: ctx.accounts.position.key(),
        owner: ctx.accounts.owner.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct PositionCreatedEvent {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub liquidity: u128,
    pub amount_0_max: u64,
    pub amount_1_max: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityIncreasedEvent {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub liquidity: u128,
    pub amount_0_max: u64,
    pub amount_1_max: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityDecreasedEvent {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub liquidity: u128,
    pub amount_0_min: u64,
    pub amount_1_min: u64,
    pub timestamp: i64,
}

#[event]
pub struct FeesCollectedEvent {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub amount_0_collected: u64,
    pub amount_1_collected: u64,
    pub timestamp: i64,
}

#[event]
pub struct PositionClosedEvent {
    pub position: Pubkey,
    pub owner: Pubkey,
    pub timestamp: i64,
}
