use anchor_lang::prelude::*;
use instructions::*;
use state::*;
mod error;
mod instructions;
mod state;


declare_id!("abcxGrLevAiSMXHnzaasyrKwU4D58w8Ab5KBA9fcrWj");

#[program]
pub mod pda_vesting {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, 
        btb: Pubkey,
        usdt: Pubkey, 
        usdc: Pubkey, 
        paypal_usd: Pubkey, 
        owner_token_receive_wallet: Pubkey,
        btb_price: u64, 
        vesting_price: u64
    ) -> Result<()> {
            process_initialize(ctx, btb, usdt, usdc, paypal_usd, owner_token_receive_wallet, btb_price, vesting_price)
    }

    pub fn update_initialize(ctx: Context<UpdateData>,
         btb: Pubkey,
         usdt: Pubkey,
         usdc: Pubkey, 
         paypal_usd: Pubkey,
         owner_token_receive_wallet: Pubkey,
         btb_price: u64,
         vesting_price: u64
        ) -> Result<()> {
            process_update_initialize(ctx, btb, usdt, usdc, paypal_usd, owner_token_receive_wallet, btb_price, vesting_price)
    }


    pub fn buy_token(ctx: Context<BuyToken>, amount: u64, token_type: u8) -> Result<()> {
        process_buy_token(ctx, amount, token_type)
    }
    
    /*
    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        transfer_admin(ctx, new_admin)
    }*/

    pub fn toggle_sale(ctx: Context<UpdateData>) -> Result<()> {
        process_toggle_sale(ctx)
    }
 
    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        process_emergency_withdraw(ctx)
    }
}







