use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};

use crate::error::CustomError;
use crate::initialize_data_account::InitializeDataAccount;

pub fn process_buy_token(ctx: Context<BuyToken>, amount: u64, token_type: u8) -> Result<()> {
    require!(amount > 0, CustomError::InvalidAmount);
    require!(token_type >= 1 && token_type <= 3, CustomError::InvalidTokenType);
    
    let btb_sale_account = &ctx.accounts.btb_sale_account;
    
    let stored_price = btb_sale_account.btb_price;

    // Calculate BTB tokens to send to user
    let btb_amount = (amount as u128)
        .checked_mul(1_000_000_000)  
        .ok_or(CustomError::CalculationError)?
        .checked_div(stored_price as u128)  
        .ok_or(CustomError::CalculationError)? as u64;

    
    require!(
        amount >= 1_000, 
        CustomError::AmountTooSmall
    );
    
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

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.owner_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;
    
    // Transfer BTB tokens to user
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
                btb_sale_account.owner_initialize_wallet.as_ref(),  
                &[ctx.bumps.btb_sale_account],
            ]],
        ),
        btb_amount,  // Changed from amount to btb_amount
    )?;
    
    Ok(())
}


#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()], bump)]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = owner_token_account.mint == user_token_account.mint,
        constraint = owner_token_account.owner == btb_sale_account.owner_token_receive_wallet
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
