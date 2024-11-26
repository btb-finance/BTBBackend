use anchor_lang::prelude::*;
use crate::error::CustomError;
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

    let program_data = ctx.accounts.program_data.try_borrow_data()?;
    let upgrade_authority = Pubkey::new_from_array(program_data[13..45].try_into().unwrap());
    require!(ctx.accounts.signer.key() == upgrade_authority, CustomError::UnauthorizedDeployer);

   require!(btb_price > 0, CustomError::ZeroBTBPrice);
   require!(vesting_price > 0, CustomError::ZeroVestingPrice);

   let sale_account = &mut ctx.accounts.btb_sale_account;

   sale_account.btb = btb;
   sale_account.usdt = usdt;
   sale_account.usdc = usdc;
   sale_account.paypal_usd = paypal_usd;
   sale_account.owner_token_receive_wallet = owner_token_receive_wallet;
   sale_account.btb_price = btb_price;
   sale_account.vesting_price = vesting_price;
   Ok(())
}
