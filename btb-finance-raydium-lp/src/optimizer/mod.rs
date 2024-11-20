use anchor_lang::prelude::*;
use crate::{
    analytics::{PositionMetrics, calculate_position_apr},
    fees::Position,
    utils::calc_fees_earned,
};

/// Fee optimization parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OptimizationParams {
    /// Minimum fee amount to trigger collection
    pub min_fee_threshold: u64,
    /// Maximum gas price willing to pay
    pub max_gas_price: u64,
    /// Target APR
    pub target_apr: f64,
    /// Collection frequency in seconds
    pub collection_frequency: i64,
}

impl Default for OptimizationParams {
    fn default() -> Self {
        Self {
            min_fee_threshold: 1_000_000, // 1 USDC
            max_gas_price: 100, // 100 GWEI
            target_apr: 20.0, // 20% APR
            collection_frequency: 24 * 60 * 60, // Daily
        }
    }
}

/// Fee collection strategy recommendation
#[derive(Debug)]
pub struct CollectionStrategy {
    /// Whether to collect fees now
    pub should_collect: bool,
    /// Optimal collection time
    pub optimal_time: i64,
    /// Expected fees at optimal time
    pub expected_fees: (u64, u64),
    /// Expected APR
    pub expected_apr: f64,
    /// Recommended gas price
    pub recommended_gas: u64,
}

/// Calculate optimal fee collection strategy
pub fn optimize_collection(
    position: &Position,
    metrics: &PositionMetrics,
    params: &OptimizationParams,
    current_gas_price: u64,
    token_a_price: u64,
    token_b_price: u64,
) -> Result<CollectionStrategy> {
    // Get current time
    let current_time = Clock::get()?.unix_timestamp;
    let time_since_last = current_time - metrics.last_collection;

    // Calculate current fees
    let current_fees_a = calc_fees_earned(
        position.liquidity,
        position.fee_growth_inside_a,
        0, // Current global growth
    )?;
    let current_fees_b = calc_fees_earned(
        position.liquidity,
        position.fee_growth_inside_b,
        0, // Current global growth
    )?;

    // Calculate fee value in USD
    let fee_value_usd = (current_fees_a as f64 * token_a_price as f64 +
        current_fees_b as f64 * token_b_price as f64) / 1e6;

    // Calculate current APR
    let current_apr = calculate_position_apr(
        position,
        metrics,
        token_a_price,
        token_b_price,
    )?;

    // Calculate transaction cost in USD
    let tx_cost_usd = (current_gas_price as f64 * 300_000.0 * token_a_price as f64) / 1e6;

    // Determine if we should collect now
    let should_collect = fee_value_usd > tx_cost_usd * 3.0 && // 3x transaction cost
        (time_since_last > params.collection_frequency || // Time threshold met
         current_apr < params.target_apr); // Below target APR

    // Calculate optimal collection time
    let fee_growth_rate = if time_since_last > 0 {
        fee_value_usd / time_since_last as f64
    } else {
        0.0
    };

    let optimal_wait_time = if fee_growth_rate > 0.0 {
        ((params.min_fee_threshold as f64 / fee_growth_rate) as i64)
            .max(params.collection_frequency)
    } else {
        params.collection_frequency
    };

    let optimal_time = current_time + optimal_wait_time;

    // Project expected fees
    let expected_fees_usd = fee_growth_rate * optimal_wait_time as f64;
    let expected_fees_a = (expected_fees_usd * 0.5 * 1e6 / token_a_price as f64) as u64;
    let expected_fees_b = (expected_fees_usd * 0.5 * 1e6 / token_b_price as f64) as u64;

    // Calculate expected APR
    let expected_apr = if current_apr > 0.0 {
        current_apr * (expected_fees_usd / fee_value_usd)
    } else {
        0.0
    };

    // Recommend gas price
    let recommended_gas = current_gas_price
        .min(params.max_gas_price)
        .max(current_gas_price / 2);

    Ok(CollectionStrategy {
        should_collect,
        optimal_time,
        expected_fees: (expected_fees_a, expected_fees_b),
        expected_apr,
        recommended_gas,
    })
}

/// Auto-compound configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AutoCompoundConfig {
    /// Whether auto-compound is enabled
    pub enabled: bool,
    /// Minimum amount to trigger auto-compound
    pub min_amount: u64,
    /// Target position range width
    pub range_width: i32,
    /// Maximum slippage tolerance
    pub max_slippage: u64,
}

impl Default for AutoCompoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_amount: 10_000_000, // 10 USDC
            range_width: 1000, // 1000 ticks
            max_slippage: 100, // 1%
        }
    }
}

/// Check if position should be auto-compounded
pub fn should_auto_compound(
    position: &Position,
    metrics: &PositionMetrics,
    config: &AutoCompoundConfig,
    token_a_price: u64,
    token_b_price: u64,
) -> bool {
    if !config.enabled {
        return false;
    }

    // Calculate total fees in USD
    let total_fees_usd = (metrics.total_fees_a as f64 * token_a_price as f64 +
        metrics.total_fees_b as f64 * token_b_price as f64) / 1e6;

    // Check if fees exceed minimum amount
    total_fees_usd >= config.min_amount as f64
}

// Example usage:
/*
    use btb_finance_clmm::optimizer::{
        OptimizationParams,
        AutoCompoundConfig,
        optimize_collection,
        should_auto_compound,
    };

    // Get optimization strategy
    let params = OptimizationParams::default();
    let strategy = optimize_collection(
        &position,
        &metrics,
        &params,
        current_gas_price,
        token_a_price,
        token_b_price,
    )?;

    if strategy.should_collect {
        msg!("Collecting fees now for optimal returns");
        collect_fees(ctx)?;
    }

    // Check auto-compound
    let config = AutoCompoundConfig::default();
    if should_auto_compound(&position, &metrics, &config, token_a_price, token_b_price) {
        msg!("Auto-compounding position for increased returns");
        // Add auto-compound logic here
    }
*/
