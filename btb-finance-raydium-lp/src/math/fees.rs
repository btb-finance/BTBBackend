use anchor_lang::prelude::*;
use crate::state::*;
use crate::utils::*;

// Fee calculation constants
pub const FEE_DENOMINATOR: u128 = 1_000_000;
pub const PROTOCOL_FEE_DENOMINATOR: u128 = 100;
pub const MIN_PROTOCOL_FEE_RATE: u8 = 1;
pub const MAX_PROTOCOL_FEE_RATE: u8 = 25;

/// Calculate the fee amount for a given trade
pub fn calculate_fee_amount(
    amount_in: u128,
    fee_rate: u32,
) -> Result<(u128, u128)> {
    let fee_amount = amount_in
        .checked_mul(fee_rate as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(FEE_DENOMINATOR)
        .ok_or(ErrorCode::MathOverflow)?;

    let amount_after_fee = amount_in
        .checked_sub(fee_amount)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok((fee_amount, amount_after_fee))
}

/// Calculate protocol fee from total fee amount
pub fn calculate_protocol_fee(
    fee_amount: u128,
    protocol_fee_rate: u8,
) -> Result<u128> {
    require!(
        protocol_fee_rate >= MIN_PROTOCOL_FEE_RATE
            && protocol_fee_rate <= MAX_PROTOCOL_FEE_RATE,
        ErrorCode::InvalidProtocolFeeRate
    );

    let protocol_fee = fee_amount
        .checked_mul(protocol_fee_rate as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(PROTOCOL_FEE_DENOMINATOR)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(protocol_fee)
}

/// Update fee growth for a position
pub fn update_position_fees(
    position: &mut LiquidityPosition,
    pool: &LiquidityPool,
    tick_lower: &Tick,
    tick_upper: &Tick,
) -> Result<(u128, u128)> {
    let (fee_growth_inside_0_x64, fee_growth_inside_1_x64) = compute_fee_growth_inside(
        pool.tick_current,
        pool.fee_growth_global_0_x64,
        pool.fee_growth_global_1_x64,
        tick_lower,
        tick_upper,
    )?;

    let uncollected_fees_0 = compute_uncollected_fees(
        position.liquidity,
        fee_growth_inside_0_x64,
        position.fee_growth_inside_0_last_x64,
    )?;

    let uncollected_fees_1 = compute_uncollected_fees(
        position.liquidity,
        fee_growth_inside_1_x64,
        position.fee_growth_inside_1_last_x64,
    )?;

    position.fee_growth_inside_0_last_x64 = fee_growth_inside_0_x64;
    position.fee_growth_inside_1_last_x64 = fee_growth_inside_1_x64;

    Ok((uncollected_fees_0, uncollected_fees_1))
}

/// Compute fee growth inside a tick range
fn compute_fee_growth_inside(
    tick_current: i32,
    fee_growth_global_0_x64: u128,
    fee_growth_global_1_x64: u128,
    tick_lower: &Tick,
    tick_upper: &Tick,
) -> Result<(u128, u128)> {
    let fee_growth_below_0_x64;
    let fee_growth_below_1_x64;
    if tick_current >= tick_lower.index {
        fee_growth_below_0_x64 = tick_lower.fee_growth_outside_0_x64;
        fee_growth_below_1_x64 = tick_lower.fee_growth_outside_1_x64;
    } else {
        fee_growth_below_0_x64 = fee_growth_global_0_x64.wrapping_sub(
            tick_lower.fee_growth_outside_0_x64
        );
        fee_growth_below_1_x64 = fee_growth_global_1_x64.wrapping_sub(
            tick_lower.fee_growth_outside_1_x64
        );
    }

    let fee_growth_above_0_x64;
    let fee_growth_above_1_x64;
    if tick_current < tick_upper.index {
        fee_growth_above_0_x64 = tick_upper.fee_growth_outside_0_x64;
        fee_growth_above_1_x64 = tick_upper.fee_growth_outside_1_x64;
    } else {
        fee_growth_above_0_x64 = fee_growth_global_0_x64.wrapping_sub(
            tick_upper.fee_growth_outside_0_x64
        );
        fee_growth_above_1_x64 = fee_growth_global_1_x64.wrapping_sub(
            tick_upper.fee_growth_outside_1_x64
        );
    }

    Ok((
        fee_growth_global_0_x64
            .wrapping_sub(fee_growth_below_0_x64)
            .wrapping_sub(fee_growth_above_0_x64),
        fee_growth_global_1_x64
            .wrapping_sub(fee_growth_below_1_x64)
            .wrapping_sub(fee_growth_above_1_x64),
    ))
}

/// Compute uncollected fees for a position
fn compute_uncollected_fees(
    liquidity: u128,
    fee_growth_inside_x64: u128,
    fee_growth_inside_last_x64: u128,
) -> Result<u128> {
    let fee_delta = fee_growth_inside_x64.wrapping_sub(fee_growth_inside_last_x64);
    
    let uncollected_fees = fee_delta
        .checked_mul(liquidity)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(1u128 << 64)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(uncollected_fees)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee_amount() {
        // Test normal case
        let (fee, amount) = calculate_fee_amount(1_000_000, 3000).unwrap(); // 0.3% fee
        assert_eq!(fee, 3000);
        assert_eq!(amount, 997000);

        // Test zero fee
        let (fee, amount) = calculate_fee_amount(1_000_000, 0).unwrap();
        assert_eq!(fee, 0);
        assert_eq!(amount, 1_000_000);

        // Test max fee
        let (fee, amount) = calculate_fee_amount(1_000_000, FEE_DENOMINATOR as u32).unwrap();
        assert_eq!(fee, 1_000_000);
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_calculate_protocol_fee() {
        // Test normal case
        let protocol_fee = calculate_protocol_fee(1_000_000, 10).unwrap(); // 10%
        assert_eq!(protocol_fee, 100_000);

        // Test min rate
        let protocol_fee = calculate_protocol_fee(1_000_000, MIN_PROTOCOL_FEE_RATE).unwrap();
        assert_eq!(protocol_fee, 10_000);

        // Test max rate
        let protocol_fee = calculate_protocol_fee(1_000_000, MAX_PROTOCOL_FEE_RATE).unwrap();
        assert_eq!(protocol_fee, 250_000);

        // Test invalid rate
        assert!(calculate_protocol_fee(1_000_000, 0).is_err());
        assert!(calculate_protocol_fee(1_000_000, 26).is_err());
    }

    #[test]
    fn test_compute_fee_growth_inside() {
        let tick_lower = Tick {
            index: -10,
            fee_growth_outside_0_x64: 100,
            fee_growth_outside_1_x64: 200,
            ..Default::default()
        };

        let tick_upper = Tick {
            index: 10,
            fee_growth_outside_0_x64: 300,
            fee_growth_outside_1_x64: 400,
            ..Default::default()
        };

        // Test current tick inside range
        let (growth_0, growth_1) = compute_fee_growth_inside(
            0,
            1000,
            2000,
            &tick_lower,
            &tick_upper,
        ).unwrap();
        assert_eq!(growth_0, 600);
        assert_eq!(growth_1, 1400);

        // Test current tick below range
        let (growth_0, growth_1) = compute_fee_growth_inside(
            -20,
            1000,
            2000,
            &tick_lower,
            &tick_upper,
        ).unwrap();
        assert_eq!(growth_0, 600);
        assert_eq!(growth_1, 1400);

        // Test current tick above range
        let (growth_0, growth_1) = compute_fee_growth_inside(
            20,
            1000,
            2000,
            &tick_lower,
            &tick_upper,
        ).unwrap();
        assert_eq!(growth_0, 600);
        assert_eq!(growth_1, 1400);
    }
}
