use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken,
};
use crate::error::CustomError;
use crate::initialize_data_account::InitializeDataAccount;

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(seeds = [b"btb-sale-account", btb_sale_account.sale_owner.as_ref()], bump)]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = owner_token_account.mint == user_token_account.mint,
        constraint = owner_token_account.owner == btb_sale_account.team_wallet
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = btb_mint_account,
        associated_token::authority = btb_sale_account
    )]
    pub btb_sale_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = btb_mint_account,
        associated_token::authority = user
    )]
    pub user_btb_account: Account<'info, TokenAccount>,

    pub btb_mint_account: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_buy_token(ctx: Context<BuyToken>, amount: i64, token_type: u8) -> Result<()> {
    
    require!(
        ctx.accounts.btb_sale_account.is_sale_active,
        CustomError::SaleNotActive
    );

   
    if amount <= 0 {
        return Err(CustomError::InvalidAmount.into());
    }

    let safe_amount = amount as u64;
    require!(token_type >= 1 && token_type <= 3, CustomError::InvalidTokenType);
    require!(safe_amount >= 1_000_000, CustomError::AmountTooSmall);  // Minimum 1 USDT

    let btb_sale_account = &ctx.accounts.btb_sale_account;

   
    let expected_mint = match token_type {
        1 => btb_sale_account.usdt,
        2 => btb_sale_account.usdc,
        3 => btb_sale_account.paypal_usd,
        _ => return Err(CustomError::InvalidTokenType.into()),
    };

   
    require!(
        ctx.accounts.user_token_account.mint == expected_mint,
        CustomError::InvalidTokenMint
    );

   
    require!(
        ctx.accounts.user_token_account.amount >= safe_amount,
        CustomError::InsufficientUserBalance
    );

   
    let btb_amount = (safe_amount as u128)
        .checked_mul(1_000_000_000)
        .ok_or(CustomError::CalculationError)?
        .checked_div(btb_sale_account.btb_price as u128)
        .ok_or(CustomError::CalculationError)? as u64;

   
    require!(btb_amount > 0, CustomError::CalculationError);

   
    require!(
        ctx.accounts.btb_sale_token_account.amount >= btb_amount,
        CustomError::InsufficientBTBBalance
    );

    
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.owner_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        safe_amount,
    )?;

    
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.btb_sale_token_account.to_account_info(),
                to: ctx.accounts.user_btb_account.to_account_info(),
                authority: ctx.accounts.btb_sale_account.to_account_info(),
            },
            &[&[
                b"btb-sale-account",
                btb_sale_account.sale_owner.as_ref(),
                &[ctx.bumps.btb_sale_account],
            ]],
        ),
        btb_amount,
    )?;

    Ok(())
}