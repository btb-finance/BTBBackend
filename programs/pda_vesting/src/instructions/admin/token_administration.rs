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



pub fn process_toggle_sale(ctx: Context<UpdateData>) -> Result<()> {
    let program_data = ctx.accounts.program_data.try_borrow_data()?;
    let upgrade_authority = Pubkey::new_from_array(program_data[13..45].try_into().unwrap());
    require!(ctx.accounts.signer.key() == upgrade_authority, CustomError::UnauthorizedDeployer);

    let sale_account = &mut ctx.accounts.btb_sale_account;
    
    
    
    sale_account.is_sale_active = !sale_account.is_sale_active;
    Ok(())
}

pub fn process_emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
    
    let program_data = ctx.accounts.program_data.try_borrow_data()?;
    let upgrade_authority = Pubkey::new_from_array(program_data[13..45].try_into().unwrap());
    require!(ctx.accounts.signer.key() == upgrade_authority, CustomError::UnauthorizedDeployer);
    
    let btb_sale_account = &ctx.accounts.btb_sale_account;
    
   
    
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