use anchor_lang::prelude::*;
use raydium_cpi::math::fees::*;

#[test]
fn test_fee_calculations() {
    // Test basic fee calculation
    let amount = 1_000_000;
    let fee_rate = 500; // 0.05%
    let fee = calculate_fee(amount, fee_rate).unwrap();
    assert_eq!(fee, 500, "Fee should be 0.05% of amount");

    // Test protocol fee calculation
    let fee = 1_000;
    let protocol_fee_rate = 10_000; // 1%
    let protocol_fee = calculate_protocol_fee(fee, protocol_fee_rate).unwrap();
    assert_eq!(protocol_fee, 10, "Protocol fee should be 1% of fee");

    // Test fee growth calculation
    let fee = 1_000;
    let liquidity = 100_000;
    let fee_growth = calculate_fee_growth(fee, liquidity).unwrap();
    assert_eq!(fee_growth, 10, "Fee growth should be fee/liquidity");
}

#[test]
fn test_fee_overflow_protection() {
    // Test fee calculation with maximum values
    let amount = u64::MAX;
    let fee_rate = 100_000; // 10%
    let result = calculate_fee(amount, fee_rate);
    assert!(result.is_ok(), "Should handle maximum amount");

    // Test protocol fee with maximum values
    let fee = u64::MAX;
    let protocol_fee_rate = 250_000; // 25%
    let result = calculate_protocol_fee(fee, protocol_fee_rate);
    assert!(result.is_ok(), "Should handle maximum fee");

    // Test fee growth with maximum values
    let fee = u128::MAX;
    let liquidity = 1;
    let result = calculate_fee_growth(fee, liquidity);
    assert!(result.is_ok(), "Should handle maximum fee growth");
}

#[test]
fn test_fee_validation() {
    // Test invalid fee rate
    let amount = 1_000_000;
    let fee_rate = 100_001; // > 10%
    let result = calculate_fee(amount, fee_rate);
    assert!(result.is_err(), "Should fail with fee rate > 10%");

    // Test invalid protocol fee rate
    let fee = 1_000;
    let protocol_fee_rate = 250_001; // > 25%
    let result = calculate_protocol_fee(fee, protocol_fee_rate);
    assert!(result.is_err(), "Should fail with protocol fee rate > 25%");

    // Test zero liquidity
    let fee = 1_000;
    let liquidity = 0;
    let result = calculate_fee_growth(fee, liquidity);
    assert!(result.is_err(), "Should fail with zero liquidity");
}

#[test]
fn test_fee_rounding() {
    // Test fee rounding down
    let amount = 999;
    let fee_rate = 500; // 0.05%
    let fee = calculate_fee(amount, fee_rate).unwrap();
    assert_eq!(fee, 0, "Small fees should round down to zero");

    // Test protocol fee rounding down
    let fee = 99;
    let protocol_fee_rate = 10_000; // 1%
    let protocol_fee = calculate_protocol_fee(fee, protocol_fee_rate).unwrap();
    assert_eq!(protocol_fee, 0, "Small protocol fees should round down to zero");

    // Test fee growth rounding
    let fee = 100;
    let liquidity = 301;
    let fee_growth = calculate_fee_growth(fee, liquidity).unwrap();
    assert_eq!(fee_growth, 0, "Small fee growth should round down to zero");
}

#[test]
fn test_cumulative_fees() {
    let mut total_fees = 0u64;
    let fee_rate = 500; // 0.05%

    // Test accumulating fees over multiple trades
    for amount in [1_000_000, 2_000_000, 3_000_000].iter() {
        let fee = calculate_fee(*amount, fee_rate).unwrap();
        total_fees = total_fees.checked_add(fee).unwrap();
    }

    assert_eq!(total_fees, 3_000, "Total fees should accumulate correctly");

    // Test protocol share of accumulated fees
    let protocol_fee_rate = 10_000; // 1%
    let protocol_share = calculate_protocol_fee(total_fees, protocol_fee_rate).unwrap();
    assert_eq!(protocol_share, 30, "Protocol share should be calculated on total fees");
}
