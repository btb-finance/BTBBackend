use anchor_lang::prelude::*;
 

#[account]
#[derive(Default)]
pub struct InitializeDataAccount {
    pub btb: Pubkey,
    pub usdt: Pubkey,
    pub usdc: Pubkey,
    pub paypal_usd: Pubkey,
    pub owner_token_receive_wallet: Pubkey,
    pub owner_initialize_wallet: Pubkey,
    pub btb_price: u64,
    pub vesting_price: u64,
}


#[account]
#[derive(InitSpace, Debug)]
pub struct VestingAccount {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub treasury_token_account: Pubkey,
    #[max_len(50)]
    pub company_name: String,
    pub treasury_bump: u8,
    pub bump: u8,
}