
use anchor_lang::prelude::*;


pub mod constants;
pub mod instructions;
pub mod state;

// pub use constants::*;
pub use instructions::*;
pub use state::*;
use crate::instructions::admin::initialize_token::Initialize;


declare_id!("4LiZrdCVJueFWYwdoiFmPqs9qKPTbFTChfKnEfcAGxBk");

#[program]
pub mod btb_project {
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
        instructions::admin::initialize_token::initialize(ctx, btb, usdt, usdc, paypal_usd, owner_token_receive_wallet, btb_price)
    }

}
