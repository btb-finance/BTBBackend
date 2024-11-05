//use crate::{Config, SEED_CONFIG_ACCOUNT};

use crate::CustomError;
use anchor_lang::prelude::*;

pub fn update_sale_status(
    ctx: Context<UpdateSale>,
    is_active: bool,
) -> Result<()> {
    require!(
        ctx.accounts.owner.key() == ctx.accounts.sale.owner,
        CustomError::UnauthorizedAdmin
    );

    let sale = &mut ctx.accounts.sale;
    sale.is_active = is_active;
    msg!("Sale status updated: {}", is_active);
    Ok(())
}

pub fn update_token_status(
    ctx: Context<UpdateSale>,
    token_index: u8,
    is_active: bool,
) -> Result<()> {
    require!(
        ctx.accounts.owner.key() == ctx.accounts.sale.owner,
        CustomError::UnauthorizedAdmin
    );

    let sale = &mut ctx.accounts.sale;
    require!(
        (token_index as usize) < sale.payment_tokens.len(),
        CustomError::InvalidPaymentToken
    );

    sale.payment_tokens[token_index as usize].is_active = is_active;
    msg!("Token {} status updated: {}", token_index, is_active);
    Ok(())
}

pub fn update_sale_params(
    ctx: Context<UpdateSale>,
    token_price: Option<u64>,
    token_vesting_price: Option<u64>,
    btb_team_wallet: Option<Pubkey>,
) -> Result<()> {
    require!(
        ctx.accounts.owner.key() == ctx.accounts.sale.owner,
        CustomError::UnauthorizedAdmin
    );

    let sale = &mut ctx.accounts.sale;

    // Update token price if provided
    if let Some(price) = token_price {
        require!(price > 0, CustomError::InvalidPrice);
        sale.token_price = price;
        msg!("Token price updated to: {}", price);
    }

    // Update vesting price if provided
    if let Some(price) = token_vesting_price {
        require!(price > 0, CustomError::InvalidPrice);
        sale.token_vesting_price = price;
        msg!("Token vesting price updated to: {}", price);
    }

    // Update team wallet if provided
    if let Some(wallet) = btb_team_wallet {
        sale.btb_team_wallet = wallet;
        msg!("Team wallet updated to: {}", wallet);
    }

    msg!("Sale parameters updated successfully");
    Ok(())
}

pub fn add_payment_token(
    ctx: Context<UpdateSale>,
    payment_token_mint: Pubkey,
) -> Result<()> {
    require!(
        ctx.accounts.owner.key() == ctx.accounts.sale.owner,
        CustomError::UnauthorizedAdmin
    );

    let sale = &mut ctx.accounts.sale;
    
    // Check if payment token already exists
    require!(
        !sale.payment_tokens.iter().any(|token| token.mint == payment_token_mint),
        CustomError::PaymentTokenExists
    );

    // Check maximum payment tokens limit
    require!(
        sale.payment_tokens.len() < 3,
        CustomError::TooManyPaymentTokens
    );

    // Add new payment token
    let new_token = PaymentToken {
        mint: payment_token_mint,
        is_active: true,
        decimals: 6, // Fixed decimals for USDC/USDT/PayPal
    };

    sale.payment_tokens.push(new_token);

    msg!("New payment token added: {}", payment_token_mint);
    msg!("Total payment tokens: {}", sale.payment_tokens.len());
    
    Ok(())
}

pub fn remove_payment_token(
    ctx: Context<UpdateSale>,
    token_index: u8,
) -> Result<()> {
    require!(
        ctx.accounts.owner.key() == ctx.accounts.sale.owner,
        CustomError::UnauthorizedAdmin
    );

    let sale = &mut ctx.accounts.sale;
    
    let index = token_index as usize;
    require!(
        index < sale.payment_tokens.len(),
        CustomError::InvalidPaymentToken
    );

    // Remove payment token
    let removed_token = sale.payment_tokens.remove(index);
    msg!("Payment token removed: {}", removed_token.mint);
    msg!("Remaining payment tokens: {}", sale.payment_tokens.len());

    Ok(())
}


#[derive(Accounts)]
pub struct UpdateSale<'info> {
    #[account(mut)]
    pub sale: Account<'info, Sale>,
    pub owner: Signer<'info>,
}

#[account]
pub struct Sale {
    pub btb_token_address: Pubkey,
    pub btb_team_wallet: Pubkey,
    pub owner: Pubkey,
    pub payment_tokens: Vec<PaymentToken>,
    pub token_price: u64,
    pub token_vesting_price: u64,
    pub is_active: bool,
}



#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PaymentToken {
    pub mint: Pubkey,
    pub is_active: bool,
    pub decimals: u8,
}

