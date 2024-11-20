use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::signature::Keypair;

use crate::utils::security::*;

#[tokio::test]
async fn test_price_range_validation() {
    // Valid price range
    let result = validate_price_range(100, 200, 150);
    assert!(result.is_ok());

    // Invalid price range (lower > upper)
    let result = validate_price_range(200, 100, 150);
    assert!(matches!(result.unwrap_err(), ErrorCode::InvalidPriceRange));

    // Price out of range
    let result = validate_price_range(100, 200, 50);
    assert!(matches!(result.unwrap_err(), ErrorCode::PriceLimitReached));
    
    let result = validate_price_range(100, 200, 250);
    assert!(matches!(result.unwrap_err(), ErrorCode::PriceLimitReached));
}

#[tokio::test]
async fn test_liquidity_validation() {
    // Valid liquidity amounts
    let result = validate_liquidity_amounts(100, 200, 50);
    assert!(result.is_ok());

    // Both amounts below minimum
    let result = validate_liquidity_amounts(10, 20, 50);
    assert!(matches!(result.unwrap_err(), ErrorCode::ZeroLiquidity));

    // One amount above minimum (valid)
    let result = validate_liquidity_amounts(10, 100, 50);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tick_spacing_validation() {
    // Valid tick spacing
    let result = validate_tick_spacing(10, -20, 20);
    assert!(result.is_ok());

    // Invalid lower tick
    let result = validate_tick_spacing(10, -25, 20);
    assert!(matches!(result.unwrap_err(), ErrorCode::InvalidTickSpacing));

    // Invalid upper tick
    let result = validate_tick_spacing(10, -20, 25);
    assert!(matches!(result.unwrap_err(), ErrorCode::InvalidTickSpacing));
}

#[tokio::test]
async fn test_authority_validation() {
    let authority = Keypair::new();
    let wrong_authority = Keypair::new();

    // Valid authority
    let result = check_authority(&authority.pubkey(), &authority.pubkey());
    assert!(result.is_ok());

    // Invalid authority
    let result = check_authority(&authority.pubkey(), &wrong_authority.pubkey());
    assert!(matches!(result.unwrap_err(), ErrorCode::InvalidAuthority));
}

#[tokio::test]
async fn test_fee_validation() {
    // Valid fee amount
    let result = validate_fee_amounts(1000, 500);
    assert!(result.is_ok());

    // Insufficient fees
    let result = validate_fee_amounts(500, 1000);
    assert!(matches!(result.unwrap_err(), ErrorCode::InsufficientProtocolFees));

    // Equal amounts (valid)
    let result = validate_fee_amounts(1000, 1000);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_combined_validations() {
    // Test multiple validations together
    let authority = Keypair::new();
    
    let result = validate_price_range(100, 200, 150)
        .and_then(|_| validate_liquidity_amounts(1000, 2000, 500))
        .and_then(|_| validate_tick_spacing(10, -20, 20))
        .and_then(|_| check_authority(&authority.pubkey(), &authority.pubkey()));
        
    assert!(result.is_ok());

    // Test with one invalid condition
    let wrong_authority = Keypair::new();
    let result = validate_price_range(100, 200, 150)
        .and_then(|_| validate_liquidity_amounts(1000, 2000, 500))
        .and_then(|_| validate_tick_spacing(10, -20, 20))
        .and_then(|_| check_authority(&authority.pubkey(), &wrong_authority.pubkey()));
        
    assert!(matches!(result.unwrap_err(), ErrorCode::InvalidAuthority));
}

// Property-based tests using proptest
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_price_range_properties(
            lower in 1u128..1000u128,
            price_diff in 1u128..1000u128,
        ) {
            let upper = lower + price_diff;
            let mid_price = lower + (price_diff / 2);
            
            let result = validate_price_range(lower, upper, mid_price);
            assert!(result.is_ok());
        }

        #[test]
        fn test_liquidity_amounts_properties(
            amount_0 in 1u64..10000u64,
            amount_1 in 1u64..10000u64,
            min_amount in 1u64..100u64,
        ) {
            let result = validate_liquidity_amounts(amount_0, amount_1, min_amount);
            assert_eq!(result.is_ok(), amount_0 >= min_amount || amount_1 >= min_amount);
        }

        #[test]
        fn test_tick_spacing_properties(
            spacing in 1u16..100u16,
            multiplier in -100i32..100i32,
        ) {
            let tick = multiplier * spacing as i32;
            let result = validate_tick_spacing(spacing, tick, tick + spacing as i32);
            assert!(result.is_ok());
        }
    }
}
