use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};

use crate::error::CustomError;
use crate::InitializeDataAccount::InitializeDataAccount;

pub fn process_initialize(ctx: Context<Initialize>, 
    btb: Pubkey,
    usdt: Pubkey, 
    usdc: Pubkey, 
    paypal_usd: Pubkey, 
    owner_token_receive_wallet: Pubkey,
    btb_price: u64, 
    vesting_price: u64
) -> Result<()> {

    require!(btb_price > 0, CustomError::ZeroBTBPrice);
    require!(vesting_price > 0, CustomError::ZeroVestingPrice);
    
    let sale_account = &mut ctx.accounts.btb_sale_account;
    sale_account.btb = btb;
    sale_account.usdt = usdt;
    sale_account.usdc = usdc;
    sale_account.paypal_usd = paypal_usd;
    sale_account.owner_token_receive_wallet = owner_token_receive_wallet;
    sale_account.owner_initialize_wallet = ctx.accounts.signer.key();
    sale_account.btb_price = btb_price;
    sale_account.vesting_price = vesting_price;
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = signer, space = 8 + 32 * 6 + 8 * 2,   
              seeds = [b"btb-sale-account", signer.key().as_ref()], bump)]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    #[account(init, payer = signer, 
              associated_token::mint = btb_mint_account,
              associated_token::authority = btb_sale_account)]
    pub btb_sale_token_account: Account<'info, TokenAccount>,
    pub btb_mint_account: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
