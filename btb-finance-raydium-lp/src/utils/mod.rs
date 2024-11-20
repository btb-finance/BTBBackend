use anchor_lang::prelude::*;
use crate::errors::ErrorCode;

/// Convert a price to a tick index
pub fn price_to_tick(price: u64) -> Result<i32> {
    let tick = (price as f64).log2() * 2f64.sqrt().log2().recip() as f64;
    Ok(tick as i32)
}

/// Convert a tick index to a price
pub fn tick_to_price(tick: i32) -> Result<u64> {
    let price = 2f64.powf(tick as f64 * 2f64.sqrt().log2());
    Ok(price as u64)
}

/// Calculate liquidity from token amounts
pub fn calc_amount_delta(
    sqrt_price: u128,
    sqrt_price_a: u64,
    sqrt_price_b: u64,
    amount_a: u64,
    amount_b: u64,
) -> Result<u128> {
    // This is a simplified calculation. In production, use more precise math
    let liquidity = (amount_a as u128)
        .checked_add(amount_b as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    Ok(liquidity)
}

/// Calculate token amounts to withdraw
pub fn calc_withdraw_amounts(
    sqrt_price: u128,
    tick_current: i32,
    liquidity: u128,
) -> Result<(u64, u64)> {
    // This is a simplified calculation. In production, use more precise math
    let amount = liquidity.checked_div(2).ok_or(ErrorCode::MathOverflow)?;
    Ok((amount as u64, amount as u64))
}

/// Calculate fees earned by a position
pub fn calc_fees_earned(
    liquidity: u128,
    fee_growth_inside: u128,
    fee_growth_global: u128,
) -> Result<u64> {
    let fee = fee_growth_global
        .checked_sub(fee_growth_inside)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(liquidity)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(u128::pow(2, 64))
        .ok_or(ErrorCode::MathOverflow)?;
    Ok(fee as u64)
}

// Example usage:
/*
    use btb_finance_clmm::utils::{price_to_tick, tick_to_price};

    // Convert price to tick
    let tick = price_to_tick(1_000_000)?; // $10.00
    msg!("Price $10.00 corresponds to tick {}", tick);

    // Convert tick to price
    let price = tick_to_price(-20)?;
    msg!("Tick -20 corresponds to price ${}", price as f64 / 100_000.0);
*/
