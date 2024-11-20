use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::{state::*, utils::*, math::*};

/// Configuration for a single swap step in the Raydium AMM V3 protocol.
/// 
/// This struct defines the parameters for executing a single swap operation
/// within a liquidity pool. It includes safety parameters like minimum output
/// and price limits to protect against adverse price movements.
///
/// # Parameters
/// * `amount_in` - The exact amount of input tokens to swap
/// * `min_amount_out` - Minimum amount of output tokens to receive (slippage protection)
/// * `sqrt_price_limit_x64` - Price limit for the swap in sqrt-price-x64 format
/// * `zero_for_one` - Direction of the swap (true for token0 -> token1, false for token1 -> token0)
///
/// # Example
/// ```rust
/// let config = SwapStepConfig {
///     amount_in: 1_000_000,
///     min_amount_out: 990_000, // 1% max slippage
///     sqrt_price_limit_x64: MIN_SQRT_PRICE_X64,
///     zero_for_one: true,
/// };
/// ```
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapStepConfig {
    /// Amount of tokens to swap in
    pub amount_in: u64,
    /// Minimum amount of tokens to receive
    pub min_amount_out: u64,
    /// Price limit for the swap
    pub sqrt_price_limit_x64: u128,
    /// Whether to swap token 0 for token 1
    pub zero_for_one: bool,
}

/// Accounts required for a single swap step
#[derive(Accounts)]
pub struct SwapStep<'info> {
    /// The pool to swap in
    #[account(
        mut,
        has_one = token_vault_0,
        has_one = token_vault_1,
    )]
    pub pool: Account<'info, LiquidityPool>,

    /// The token account to swap from
    #[account(mut)]
    pub token_account_in: Account<'info, TokenAccount>,

    /// The token account to swap to
    #[account(mut)]
    pub token_account_out: Account<'info, TokenAccount>,

    /// The pool's token 0 vault
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,

    /// The pool's token 1 vault
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,

    /// The tick array containing the current tick
    #[account(mut)]
    pub tick_array: Account<'info, TickArray>,

    /// The pool authority PDA
    /// CHECK: PDA owned by the program
    #[account(
        seeds = [
            seeds::POOL_AUTHORITY,
            pool.key().as_ref(),
        ],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    /// The token program
    pub token_program: Program<'info, Token>,
}

/// Configuration for executing swaps through multiple pools (router swap).
///
/// This struct defines parameters for performing multi-hop swaps through
/// a series of Raydium AMM V3 pools to achieve better pricing or enable
/// swaps between token pairs that don't have direct pools.
///
/// # Parameters
/// * `amount_in` - Initial amount of input tokens
/// * `min_amount_out` - Minimum final output amount across all hops
/// * `route` - Vector of pool addresses defining the swap route
///
/// # Example
/// ```rust
/// let config = RouterConfig {
///     amount_in: 1_000_000,
///     min_amount_out: 980_000, // 2% max slippage
///     route: vec![pool1, pool2], // USDC -> SOL -> RAY
/// };
/// ```
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RouterConfig {
    /// Amount of tokens to swap in
    pub amount_in: u64,
    /// Minimum amount of tokens to receive
    pub min_amount_out: u64,
    /// List of pools to swap through
    pub route: Vec<Pubkey>,
}

/// Accounts required for a router swap
#[derive(Accounts)]
pub struct RouterSwap<'info> {
    /// The authority performing the swap
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The token account to swap from
    #[account(mut)]
    pub token_account_in: Account<'info, TokenAccount>,

    /// The token account to swap to
    #[account(mut)]
    pub token_account_out: Account<'info, TokenAccount>,

    /// The token program
    pub token_program: Program<'info, Token>,
}

/// Execute a single swap step within a Raydium AMM V3 pool.
///
/// This function performs a swap of tokens within a single liquidity pool,
/// handling all necessary price calculations, fee collection, and token transfers.
///
/// # Arguments
/// * `ctx` - Context containing all required accounts
/// * `config` - Swap configuration parameters
///
/// # Returns
/// * `Result<u64>` - Amount of tokens received from the swap
///
/// # Errors
/// * Returns error if slippage tolerance is exceeded
/// * Returns error if price limit is breached
/// * Returns error if insufficient liquidity
///
/// # Example
/// ```rust
/// let amount_out = swap_step(ctx, config)?;
/// msg!("Received {} tokens", amount_out);
/// ```
pub fn swap_step(
    ctx: Context<SwapStep>,
    config: SwapStepConfig,
) -> Result<u64> {
    let pool = &mut ctx.accounts.pool;
    let tick_array = &mut ctx.accounts.tick_array;

    // Validate price limit
    require!(
        config.sqrt_price_limit_x64 >= MIN_SQRT_PRICE_X64
            && config.sqrt_price_limit_x64 <= MAX_SQRT_PRICE_X64,
        ErrorCode::InvalidPriceLimit
    );

    // Calculate amounts
    let (amount_in, amount_out, new_sqrt_price_x64, new_tick) = compute_swap(
        pool.sqrt_price_x64,
        pool.tick_current,
        pool.liquidity,
        config.amount_in,
        config.zero_for_one,
        config.sqrt_price_limit_x64,
    )?;

    // Verify slippage
    require!(
        amount_out >= config.min_amount_out,
        ErrorCode::ExcessiveSlippage
    );

    // Update pool state
    pool.sqrt_price_x64 = new_sqrt_price_x64;
    pool.tick_current = new_tick;

    // Calculate and update fees
    let fee_amount = calculate_fee(amount_in, pool.fee_rate)?;
    let protocol_fee = calculate_protocol_fee(fee_amount, pool.protocol_fee_rate)?;
    
    if config.zero_for_one {
        pool.fee_growth_global_0_x64 = pool.fee_growth_global_0_x64
            .checked_add(calculate_fee_growth(fee_amount, pool.liquidity)?)
            .ok_or(ErrorCode::MathOverflow)?;
        pool.protocol_fees_token_0 = pool.protocol_fees_token_0
            .checked_add(protocol_fee)
            .ok_or(ErrorCode::MathOverflow)?;
    } else {
        pool.fee_growth_global_1_x64 = pool.fee_growth_global_1_x64
            .checked_add(calculate_fee_growth(fee_amount, pool.liquidity)?)
            .ok_or(ErrorCode::MathOverflow)?;
        pool.protocol_fees_token_1 = pool.protocol_fees_token_1
            .checked_add(protocol_fee)
            .ok_or(ErrorCode::MathOverflow)?;
    }

    // Transfer tokens
    if config.zero_for_one {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_account_in.to_account_info(),
                    to: ctx.accounts.token_vault_0.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
            ),
            amount_in,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_vault_1.to_account_info(),
                    to: ctx.accounts.token_account_out.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                &[&[
                    seeds::POOL_AUTHORITY,
                    pool.key().as_ref(),
                    &[*ctx.bumps.get("pool_authority").unwrap()],
                ]],
            ),
            amount_out,
        )?;
    } else {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_account_in.to_account_info(),
                    to: ctx.accounts.token_vault_1.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
            ),
            amount_in,
        )?;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_vault_0.to_account_info(),
                    to: ctx.accounts.token_account_out.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                &[&[
                    seeds::POOL_AUTHORITY,
                    pool.key().as_ref(),
                    &[*ctx.bumps.get("pool_authority").unwrap()],
                ]],
            ),
            amount_out,
        )?;
    }

    Ok(amount_out)
}

/// Execute a multi-hop swap through multiple Raydium AMM V3 pools.
///
/// This function performs a series of swaps through multiple pools to
/// achieve better pricing or enable swaps between token pairs that
/// don't have direct pools.
///
/// # Arguments
/// * `ctx` - Context containing all required accounts
/// * `config` - Router configuration parameters
///
/// # Returns
/// * `Result<u64>` - Final amount of tokens received
///
/// # Errors
/// * Returns error if any pool in the route fails
/// * Returns error if final amount is below minimum
/// * Returns error if insufficient liquidity
///
/// # Example
/// ```rust
/// let final_amount = router_swap(ctx, config)?;
/// msg!("Received {} tokens after multi-hop swap", final_amount);
/// ```
pub fn router_swap(
    ctx: Context<RouterSwap>,
    config: RouterConfig,
) -> Result<u64> {
    let mut amount_in = config.amount_in;
    let mut total_amount_out = 0;

    // Perform swaps through each pool in the route
    for (i, pool_key) in config.route.iter().enumerate() {
        let pool = Account::<LiquidityPool>::try_from(&ctx.accounts.pool)?;
        let tick_array = Account::<TickArray>::try_from(&ctx.accounts.tick_array)?;

        let min_amount_out = if i == config.route.len() - 1 {
            config.min_amount_out
        } else {
            0 // No slippage check for intermediate swaps
        };

        let sqrt_price_limit_x64 = if i % 2 == 0 {
            MIN_SQRT_PRICE_X64 // Swap token 0 for token 1
        } else {
            MAX_SQRT_PRICE_X64 // Swap token 1 for token 0
        };

        let step_config = SwapStepConfig {
            amount_in,
            min_amount_out,
            sqrt_price_limit_x64,
            zero_for_one: i % 2 == 0,
        };

        let amount_out = swap_step(
            ctx.accounts
                .try_accounts(SwapStep {
                    pool,
                    token_account_in: ctx.accounts.token_account_in.clone(),
                    token_account_out: ctx.accounts.token_account_out.clone(),
                    token_vault_0: pool.token_vault_0,
                    token_vault_1: pool.token_vault_1,
                    tick_array,
                    pool_authority: pool.authority,
                    token_program: ctx.accounts.token_program.clone(),
                })?,
            step_config,
        )?;

        amount_in = amount_out;
        total_amount_out = amount_out;
    }

    Ok(total_amount_out)
}

/// Minimum allowed sqrt-price-x64 value for price limit validation
const MIN_SQRT_PRICE_X64: u128 = 4_295_048_016;

/// Maximum allowed sqrt-price-x64 value for price limit validation
const MAX_SQRT_PRICE_X64: u128 = 79_226_673_515_401_279_992_447_579_055;
