use anchor_lang::{prelude::*,  solana_program::bpf_loader_upgradeable};
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};
use crate::initialize_data_account::InitializeDataAccount;

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()], bump)]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
        /// CHECK: Program data account containing upgrade authority
    #[account(
            constraint = program_data.owner == &bpf_loader_upgradeable::ID
     )]
    pub program_data: AccountInfo<'info>,

        
    #[account(
        mut,
        associated_token::mint = btb_mint_account,
        associated_token::authority = btb_sale_account
    )]
    pub btb_sale_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = owner_btb_account.mint == btb_mint_account.key(),
        constraint = owner_btb_account.owner == signer.key()
    )]
    pub owner_btb_account: Account<'info, TokenAccount>,
    
    pub btb_mint_account: Account<'info, Mint>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
