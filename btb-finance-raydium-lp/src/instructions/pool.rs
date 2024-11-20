use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

// Constants
const MAX_FEE_RATE: u32 = 100_000; // 10%
const MIN_PROTOCOL_FEE_RATE: u8 = 1; // 1%
const MAX_PROTOCOL_FEE_RATE: u8 = 25; // 25%
const MAX_TICK_SPACING: u16 = 100;
const MIN_SQRT_PRICE_X64: u128 = 4295048016;
const MIN_TICK: i32 = -443636;

/// Pool configuration parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PoolConfig {
    /// Fee rate in hundredths of a bip (0.0001%)
    pub fee_rate: u32,
    /// Protocol fee rate as percentage of fee rate
    pub protocol_fee_rate: u8,
    /// Tick spacing for the pool
    pub tick_spacing: u16,
}

/// Instruction context for creating a new liquidity pool
#[derive(Accounts)]
#[instruction(pool_config: PoolConfig)]
pub struct CreatePool<'info> {
    /// The authority that will own and manage the pool
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The pool account to be created
    #[account(
        init,
        payer = owner,
        space = LiquidityPool::LEN,
        seeds = [
            seeds::POOL,
            token_mint_0.key().as_ref(),
            token_mint_1.key().as_ref(),
            pool_config.fee_rate.to_le_bytes().as_ref(),
        ],
        bump
    )]
    pub pool: Account<'info, LiquidityPool>,

    /// The mint for token 0
    pub token_mint_0: Account<'info, Mint>,

    /// The mint for token 1
    pub token_mint_1: Account<'info, Mint>,

    /// The vault for token 0
    #[account(
        init,
        payer = owner,
        token::mint = token_mint_0,
        token::authority = pool_authority,
    )]
    pub token_vault_0: Account<'info, TokenAccount>,

    /// The vault for token 1
    #[account(
        init,
        payer = owner,
        token::mint = token_mint_1,
        token::authority = pool_authority,
    )]
    pub token_vault_1: Account<'info, TokenAccount>,

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

    /// The system program
    pub system_program: Program<'info, System>,
}

/// Create a new liquidity pool with the given configuration
pub fn create_pool(
    ctx: Context<CreatePool>,
    pool_config: PoolConfig,
) -> Result<()> {
    // Validate pool configuration
    require!(
        pool_config.fee_rate <= MAX_FEE_RATE,
        ErrorCode::InvalidFeeRate
    );
    require!(
        pool_config.protocol_fee_rate >= MIN_PROTOCOL_FEE_RATE
            && pool_config.protocol_fee_rate <= MAX_PROTOCOL_FEE_RATE,
        ErrorCode::InvalidProtocolFeeRate
    );
    require!(
        pool_config.tick_spacing > 0 && pool_config.tick_spacing <= MAX_TICK_SPACING,
        ErrorCode::InvalidTickSpacing
    );

    // Ensure token mints are different and ordered
    require!(
        ctx.accounts.token_mint_0.key() < ctx.accounts.token_mint_1.key(),
        ErrorCode::InvalidTokenMintOrder
    );

    let pool = &mut ctx.accounts.pool;

    // Initialize pool state
    pool.token_mint_0 = ctx.accounts.token_mint_0.key();
    pool.token_mint_1 = ctx.accounts.token_mint_1.key();
    pool.token_vault_0 = ctx.accounts.token_vault_0.key();
    pool.token_vault_1 = ctx.accounts.token_vault_1.key();
    pool.fee_rate = pool_config.fee_rate;
    pool.protocol_fee_rate = pool_config.protocol_fee_rate;
    pool.tick_spacing = pool_config.tick_spacing;
    pool.liquidity = 0;
    pool.sqrt_price_x64 = MIN_SQRT_PRICE_X64;
    pool.tick_current = MIN_TICK;
    pool.fee_growth_global_0_x64 = 0;
    pool.fee_growth_global_1_x64 = 0;
    pool.protocol_fees_0 = 0;
    pool.protocol_fees_1 = 0;
    pool.unlocked = true;

    Ok(())
}

/// Instruction context for updating pool parameters
#[derive(Accounts)]
pub struct UpdatePool<'info> {
    /// The authority that owns the pool
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The pool to update
    #[account(
        mut,
        has_one = owner @ ErrorCode::InvalidPoolOwner,
    )]
    pub pool: Account<'info, LiquidityPool>,
}

/// Update pool parameters
pub fn update_pool(
    ctx: Context<UpdatePool>,
    fee_rate: Option<u32>,
    protocol_fee_rate: Option<u8>,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // Update fee rate if provided
    if let Some(new_fee_rate) = fee_rate {
        require!(
            new_fee_rate <= MAX_FEE_RATE,
            ErrorCode::InvalidFeeRate
        );
        pool.fee_rate = new_fee_rate;
    }

    // Update protocol fee rate if provided
    if let Some(new_protocol_fee_rate) = protocol_fee_rate {
        require!(
            new_protocol_fee_rate >= MIN_PROTOCOL_FEE_RATE
                && new_protocol_fee_rate <= MAX_PROTOCOL_FEE_RATE,
            ErrorCode::InvalidProtocolFeeRate
        );
        pool.protocol_fee_rate = new_protocol_fee_rate;
    }

    Ok(())
}

/// Instruction context for collecting protocol fees
#[derive(Accounts)]
pub struct CollectProtocolFees<'info> {
    /// The authority that owns the pool
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The pool to collect fees from
    #[account(
        mut,
        has_one = owner @ ErrorCode::InvalidPoolOwner,
        has_one = token_vault_0,
        has_one = token_vault_1,
    )]
    pub pool: Account<'info, LiquidityPool>,

    /// The vault for token 0
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,

    /// The vault for token 1
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,

    /// The recipient account for token 0
    #[account(mut)]
    pub recipient_token_account_0: Account<'info, TokenAccount>,

    /// The recipient account for token 1
    #[account(mut)]
    pub recipient_token_account_1: Account<'info, TokenAccount>,

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

/// Collect accumulated protocol fees
pub fn collect_protocol_fees(ctx: Context<CollectProtocolFees>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let amount_0 = pool.protocol_fees_0;
    let amount_1 = pool.protocol_fees_1;

    // Reset protocol fees
    pool.protocol_fees_0 = 0;
    pool.protocol_fees_1 = 0;

    // Transfer token 0 fees
    if amount_0 > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_vault_0.to_account_info(),
                    to: ctx.accounts.recipient_token_account_0.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                &[&[
                    seeds::POOL_AUTHORITY,
                    pool.key().as_ref(),
                    &[*ctx.bumps.get("pool_authority").unwrap()],
                ]],
            ),
            amount_0,
        )?;
    }

    // Transfer token 1 fees
    if amount_1 > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.token_vault_1.to_account_info(),
                    to: ctx.accounts.recipient_token_account_1.to_account_info(),
                    authority: ctx.accounts.pool_authority.to_account_info(),
                },
                &[&[
                    seeds::POOL_AUTHORITY,
                    pool.key().as_ref(),
                    &[*ctx.bumps.get("pool_authority").unwrap()],
                ]],
            ),
            amount_1,
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePoolContext<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    /// CHECK: Validated in CPI
    #[account(mut)]
    pub pool_state: AccountInfo<'info>,
    /// CHECK: Validated in CPI
    pub pool_authority: AccountInfo<'info>,
    #[account(mut)]
    pub token_a_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_b_account: Account<'info, TokenAccount>,
    /// CHECK: Validated in CPI
    #[account(mut)]
    pub token_a_vault: AccountInfo<'info>,
    /// CHECK: Validated in CPI
    #[account(mut)]
    pub token_b_vault: AccountInfo<'info>,
    /// CHECK: Validated in CPI
    #[account(mut)]
    pub pool_token_mint: AccountInfo<'info>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn initialize_pool(
    ctx: Context<InitializePoolContext>,
    amount_a: u64,
    amount_b: u64,
) -> Result<()> {
    // Implementation for initializing a pool
    // This is where you would make the CPI call to Raydium's initialize_pool instruction
    Ok(())
}
