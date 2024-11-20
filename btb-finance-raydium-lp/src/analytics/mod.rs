use anchor_lang::prelude::*;
use crate::{
    fees::{CollectFeesAccounts, Position},
    utils::calc_fees_earned,
};

/// Fee collection event for analytics
#[event]
pub struct FeeCollectionEvent {
    /// Position owner
    pub owner: Pubkey,
    /// Pool address
    pub pool: Pubkey,
    /// Position address
    pub position: Pubkey,
    /// Amount of token A collected
    pub amount_a: u64,
    /// Amount of token B collected
    pub amount_b: u64,
    /// Timestamp of collection
    pub timestamp: i64,
    /// Gas used for collection
    pub gas_used: u64,
    /// Transaction cost in lamports
    pub tx_cost: u64,
}

/// Position metrics for analytics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PositionMetrics {
    /// Total fees collected (token A)
    pub total_fees_a: u64,
    /// Total fees collected (token B)
    pub total_fees_b: u64,
    /// Number of fee collections
    pub collection_count: u64,
    /// Last collection timestamp
    pub last_collection: i64,
    /// Average fees per collection (token A)
    pub avg_fees_a: u64,
    /// Average fees per collection (token B)
    pub avg_fees_b: u64,
    /// Total gas used
    pub total_gas_used: u64,
    /// Total transaction costs
    pub total_tx_costs: u64,
    /// Best collection (highest fees)
    pub best_collection: FeeCollectionStats,
    /// Worst collection (lowest fees)
    pub worst_collection: FeeCollectionStats,
    /// Collection frequency in seconds
    pub avg_collection_frequency: i64,
    /// Success rate (successful collections / total attempts)
    pub success_rate: f64,
}

/// Statistics for a single fee collection
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FeeCollectionStats {
    /// Amount of token A collected
    pub amount_a: u64,
    /// Amount of token B collected
    pub amount_b: u64,
    /// Timestamp of collection
    pub timestamp: i64,
    /// Gas used
    pub gas_used: u64,
    /// Transaction cost
    pub tx_cost: u64,
    /// USD value at collection time
    pub usd_value: f64,
}

impl PositionMetrics {
    /// Create new position metrics
    pub fn new() -> Self {
        Self {
            total_fees_a: 0,
            total_fees_b: 0,
            collection_count: 0,
            last_collection: 0,
            avg_fees_a: 0,
            avg_fees_b: 0,
            total_gas_used: 0,
            total_tx_costs: 0,
            best_collection: FeeCollectionStats {
                amount_a: 0,
                amount_b: 0,
                timestamp: 0,
                gas_used: 0,
                tx_cost: 0,
                usd_value: 0.0,
            },
            worst_collection: FeeCollectionStats {
                amount_a: u64::MAX,
                amount_b: u64::MAX,
                timestamp: 0,
                gas_used: 0,
                tx_cost: 0,
                usd_value: f64::MAX,
            },
            avg_collection_frequency: 0,
            success_rate: 1.0,
        }
    }

    /// Update metrics after fee collection
    pub fn update(
        &mut self,
        amount_a: u64,
        amount_b: u64,
        timestamp: i64,
        gas_used: u64,
        tx_cost: u64,
        token_a_price: u64,
        token_b_price: u64,
    ) {
        // Update totals
        self.total_fees_a = self.total_fees_a.saturating_add(amount_a);
        self.total_fees_b = self.total_fees_b.saturating_add(amount_b);
        self.total_gas_used = self.total_gas_used.saturating_add(gas_used);
        self.total_tx_costs = self.total_tx_costs.saturating_add(tx_cost);
        
        // Update collection count
        self.collection_count = self.collection_count.saturating_add(1);
        
        // Update averages
        self.avg_fees_a = self.total_fees_a.checked_div(self.collection_count).unwrap_or(0);
        self.avg_fees_b = self.total_fees_b.checked_div(self.collection_count).unwrap_or(0);
        
        // Calculate USD value
        let usd_value = (amount_a as f64 * token_a_price as f64 +
            amount_b as f64 * token_b_price as f64) / 1e6;
        
        // Update best/worst collections
        let stats = FeeCollectionStats {
            amount_a,
            amount_b,
            timestamp,
            gas_used,
            tx_cost,
            usd_value,
        };
        
        if usd_value > self.best_collection.usd_value {
            self.best_collection = stats.clone();
        }
        if usd_value < self.worst_collection.usd_value {
            self.worst_collection = stats;
        }
        
        // Update collection frequency
        if self.last_collection > 0 {
            let frequency = timestamp - self.last_collection;
            self.avg_collection_frequency = if self.collection_count > 1 {
                (self.avg_collection_frequency * (self.collection_count - 1) as i64 + frequency) /
                    self.collection_count as i64
            } else {
                frequency
            };
        }
        
        self.last_collection = timestamp;
    }

    /// Record failed collection attempt
    pub fn record_failure(&mut self) {
        self.success_rate = (self.success_rate * self.collection_count as f64) /
            (self.collection_count + 1) as f64;
    }
}

/// Calculate ROI metrics for a position
pub fn calculate_roi_metrics(
    position: &Position,
    metrics: &PositionMetrics,
    token_a_price: u64,
    token_b_price: u64,
) -> Result<ROIMetrics> {
    // Calculate total fees in USD
    let total_fees_usd = (metrics.total_fees_a as f64 * token_a_price as f64 +
        metrics.total_fees_b as f64 * token_b_price as f64) / 1e6;

    // Calculate total costs in USD
    let total_costs_usd = metrics.total_tx_costs as f64 * token_a_price as f64 / 1e6;

    // Calculate net profit
    let net_profit_usd = total_fees_usd - total_costs_usd;

    // Calculate position value
    let position_value_usd = position.liquidity as f64 *
        (token_a_price + token_b_price) as f64 / 1e6;

    // Calculate ROI
    let roi = if position_value_usd > 0.0 {
        net_profit_usd / position_value_usd * 100.0
    } else {
        0.0
    };

    // Calculate profit margin
    let profit_margin = if total_fees_usd > 0.0 {
        net_profit_usd / total_fees_usd * 100.0
    } else {
        0.0
    };

    Ok(ROIMetrics {
        total_fees_usd,
        total_costs_usd,
        net_profit_usd,
        roi,
        profit_margin,
        avg_cost_per_collection: total_costs_usd / metrics.collection_count as f64,
        success_rate: metrics.success_rate,
    })
}

/// Return on investment metrics
#[derive(Debug)]
pub struct ROIMetrics {
    /// Total fees earned in USD
    pub total_fees_usd: f64,
    /// Total costs in USD
    pub total_costs_usd: f64,
    /// Net profit in USD
    pub net_profit_usd: f64,
    /// Return on investment percentage
    pub roi: f64,
    /// Profit margin percentage
    pub profit_margin: f64,
    /// Average cost per collection
    pub avg_cost_per_collection: f64,
    /// Success rate
    pub success_rate: f64,
}

/// Calculate estimated APR for a position
pub fn calculate_position_apr(
    position: &Position,
    metrics: &PositionMetrics,
    token_a_price: u64,
    token_b_price: u64,
) -> Result<f64> {
    // Convert fees to USD value
    let fees_usd = (metrics.total_fees_a as f64 * token_a_price as f64 +
        metrics.total_fees_b as f64 * token_b_price as f64) / 1e6; // Assuming 6 decimals

    // Calculate time period in years
    let time_period = (Clock::get()?.unix_timestamp - metrics.last_collection) as f64 / (365.0 * 24.0 * 60.0 * 60.0);

    // Calculate position value in USD
    let position_value = position.liquidity as f64 * (token_a_price + token_b_price) as f64 / 1e6;

    // Calculate APR
    let apr = if position_value > 0.0 && time_period > 0.0 {
        (fees_usd / position_value / time_period) * 100.0
    } else {
        0.0
    };

    Ok(apr)
}

/// Get optimal fee collection threshold
pub fn get_collection_threshold(
    position: &Position,
    metrics: &PositionMetrics,
    gas_price: u64,
) -> Result<(u64, u64)> {
    // Estimate transaction cost
    let tx_cost = gas_price.checked_mul(300_000).unwrap_or(0); // Assuming 300k gas units

    // Calculate average fee growth per day
    let avg_daily_a = if metrics.collection_count > 0 {
        metrics.total_fees_a
            .checked_div(metrics.collection_count)
            .unwrap_or(0)
    } else {
        0
    };

    let avg_daily_b = if metrics.collection_count > 0 {
        metrics.total_fees_b
            .checked_div(metrics.collection_count)
            .unwrap_or(0)
    } else {
        0
    };

    // Set threshold to 3x transaction cost
    let threshold_a = tx_cost.checked_mul(3).unwrap_or(avg_daily_a);
    let threshold_b = tx_cost.checked_mul(3).unwrap_or(avg_daily_b);

    Ok((threshold_a, threshold_b))
}

// Example usage:
/*
    use btb_finance_clmm::analytics::{
        FeeCollectionEvent,
        PositionMetrics,
        calculate_roi_metrics,
    };

    // Track fee collection
    let mut metrics = PositionMetrics::new();
    
    // Update metrics with collection results
    metrics.update(
        fees_a,
        fees_b,
        Clock::get()?.unix_timestamp,
        gas_used,
        tx_cost,
        token_a_price,
        token_b_price,
    );

    // Calculate ROI metrics
    let roi = calculate_roi_metrics(
        &position,
        &metrics,
        token_a_price,
        token_b_price,
    )?;

    msg!(
        "Position ROI: {}%, Profit Margin: {}%, Success Rate: {}%",
        roi.roi,
        roi.profit_margin,
        roi.success_rate * 100.0
    );
*/
