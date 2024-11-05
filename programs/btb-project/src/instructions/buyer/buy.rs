use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};

use crate::InitializeDataAccount::InitializeDataAccount;


#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()],
                     bump)]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = owner_token_account.mint == user_token_account.mint,
        constraint = owner_token_account.owner == btb_sale_account.owner_token_receive_wallet
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = btb_mint_account,
        associated_token::authority = btb_sale_account
    )]
    pub btb_sale_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = btb_mint_account,
        associated_token::authority = user
    )]
    pub user_btb_account: Account<'info, TokenAccount>,
    pub btb_mint_account: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
