use anchor_lang::prelude::*;
use crate::errors::ErrorCode;

pub fn validate_price_range(
    lower_price: u128,
    upper_price: u128,
    current_price: u128,
) -> Result<()> {
    require!(lower_price < upper_price, ErrorCode::InvalidPriceRange);
    require!(
        current_price >= lower_price && current_price <= upper_price,
        ErrorCode::PriceLimitReached
    );
    Ok(())
}

pub fn validate_liquidity_amounts(
    amount_0: u64,
    amount_1: u64,
    min_amount: u64,
) -> Result<()> {
    require!(
        amount_0 >= min_amount || amount_1 >= min_amount,
        ErrorCode::ZeroLiquidity
    );
    Ok(())
}

pub fn validate_tick_spacing(
    tick_spacing: u16,
    tick_lower: i32,
    tick_upper: i32,
) -> Result<()> {
    require!(
        tick_lower % (tick_spacing as i32) == 0 && tick_upper % (tick_spacing as i32) == 0,
        ErrorCode::InvalidTickSpacing
    );
    Ok(())
}

pub fn check_authority(
    authority: &Pubkey,
    expected: &Pubkey,
) -> Result<()> {
    require!(authority == expected, ErrorCode::InvalidAuthority);
    Ok(())
}

pub fn validate_fee_amounts(
    collected_amount: u64,
    requested_amount: u64,
) -> Result<()> {
    require!(
        collected_amount >= requested_amount,
        ErrorCode::InsufficientProtocolFees
    );
    Ok(())
}
