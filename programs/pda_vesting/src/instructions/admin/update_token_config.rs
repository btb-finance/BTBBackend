use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};

use crate::error::CustomError;
use crate::initialize_data_account::InitializeDataAccount;
use crate::update_data::UpdateData;

pub fn process_update_initialize(ctx: Context<UpdateData>,
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
   require!(ctx.accounts.signer.key() == sale_account.owner_initialize_wallet, CustomError::Unauthorized);

   sale_account.btb = btb;
   sale_account.usdt = usdt;
   sale_account.usdc = usdc;
   sale_account.paypal_usd = paypal_usd;
   sale_account.owner_token_receive_wallet = owner_token_receive_wallet;
   sale_account.btb_price = btb_price;
   sale_account.vesting_price = vesting_price;
   Ok(())
}

