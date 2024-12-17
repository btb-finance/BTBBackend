use anchor_lang::prelude::*;
pub mod instructions;
use instructions::*;

declare_id!("FZVj5H8DrZ6jwRpEwmN4reLN1khdUEKiD54oBkcjfYfN");

#[program]
pub mod btb_lp {
    use super::*;

    pub fn proxy_initialize(
        ctx: Context<ProxyInitialize>,
        sqrt_price_x64: u128,
        open_time: u64,
    ) -> Result<()> {
        instructions::proxy_initialize(ctx, sqrt_price_x64, open_time)
    }

    pub fn proxy_open_position<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, ProxyOpenPosition<'info>>,
        tick_lower_index: i32,
        tick_upper_index: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
        liquidity: u128,
        amount_0_max: u64,
        amount_1_max: u64,
        with_matedata: bool,
        // base_flag: Option<bool>,
    ) -> Result<()> {
        instructions::proxy_open_position(
            ctx,
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            liquidity,
            amount_0_max,
            amount_1_max,
            with_matedata,
            None,
        )
    }
}

