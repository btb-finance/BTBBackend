use anchor_lang::prelude::*;
use crate::InitializeDataAccount;

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(
        mut,
        seeds = [b"btb-sale-account", signer.key().as_ref()],
        bump
    )]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
