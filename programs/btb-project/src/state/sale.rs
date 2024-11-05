use anchor_lang::prelude::*;

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
