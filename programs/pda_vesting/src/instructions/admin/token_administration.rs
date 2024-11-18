use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};

use crate::transfer_admin::TransferAdmin;
use crate::error::CustomError;
use crate::initialize_data_account::InitializeDataAccount;
use crate::emergency_withdraw::EmergencyWithdraw;
use crate::update_data::UpdateData;

pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
    require!(new_admin != Pubkey::default(), CustomError::InvalidNewAdmin);
    
    let sale_account = &mut ctx.accounts.btb_sale_account;
    
    // Verify current signer is the admin
    require!(
        ctx.accounts.signer.key() == sale_account.owner_initialize_wallet,
        CustomError::Unauthorized
    );

    sale_account.owner_initialize_wallet = new_admin;
    Ok(())
}

pub fn process_toggle_sale(ctx: Context<UpdateData>) -> Result<()> {
    let sale_account = &mut ctx.accounts.btb_sale_account;
    
    // Only owner can toggle sale status
    require!(
        ctx.accounts.signer.key() == sale_account.owner_initialize_wallet
        && sale_account.owner_initialize_wallet == *ctx.program_id,
        CustomError::Unauthorized
    );
    
    sale_account.is_sale_active = !sale_account.is_sale_active;
    Ok(())
}

pub fn process_emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
    let btb_sale_account = &ctx.accounts.btb_sale_account;
    
    // Enhanced owner validation
    require!(
        ctx.accounts.signer.key() == btb_sale_account.owner_initialize_wallet
        && btb_sale_account.owner_initialize_wallet == *ctx.program_id,
        CustomError::Unauthorized
    );
    
    let balance = ctx.accounts.btb_sale_token_account.amount;
    require!(balance > 0, CustomError::NoTokensToWithdraw);

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.btb_sale_token_account.to_account_info(),
                to: ctx.accounts.owner_btb_account.to_account_info(),
                authority: ctx.accounts.btb_sale_account.to_account_info(),
            },
            &[&[
                b"btb-sale-account",
                btb_sale_account.owner_initialize_wallet.as_ref(),
                &[ctx.bumps.btb_sale_account],
            ]],
        ),
        balance,
    )?;
    
    Ok(())
}