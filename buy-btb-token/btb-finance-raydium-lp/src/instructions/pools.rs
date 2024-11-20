use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use raydium_clmm_sdk::states::Pool;

use crate::{InitializePool, errors::ErrorCode};

pub fn initialize_pool(
    ctx: Context<InitializePool>,
    tick_spacing: u16,
    initial_price: u128,
) -> Result<()> {
    // Validate parameters
    require!(
        tick_spacing > 0 && tick_spacing <= 100,
        ErrorCode::InvalidParameters
    );

    require!(initial_price > 0, ErrorCode::InvalidParameters);

    // Create CPI context
    let cpi_accounts = raydium_clmm_sdk::accounts::InitializePool {
        pool: ctx.accounts.pool.to_account_info(),
        token_vault_0: ctx.accounts.token_vault_0.to_account_info(),
        token_vault_1: ctx.accounts.token_vault_1.to_account_info(),
        observation_state: ctx.accounts.observation_state.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_program = ctx.accounts.raydium_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute CPI call
    raydium_clmm_sdk::cpi::initialize_pool(
        cpi_ctx,
        tick_spacing,
        initial_price,
    ).map_err(|_| ErrorCode::CpiCallFailed)?;

    // Emit pool creation event
    emit!(PoolCreatedEvent {
        pool: ctx.accounts.pool.key(),
        token_vault_0: ctx.accounts.token_vault_0.key(),
        token_vault_1: ctx.accounts.token_vault_1.key(),
        tick_spacing,
        initial_price,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct PoolCreatedEvent {
    pub pool: Pubkey,
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,
    pub tick_spacing: u16,
    pub initial_price: u128,
    pub timestamp: i64,
}
