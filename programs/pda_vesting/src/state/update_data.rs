use anchor_lang::prelude::*;
use crate::initialize_data_account::InitializeDataAccount;

#[derive(Accounts)]
pub struct UpdateData<'info> {
    #[account(
        mut,
        seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()],
        bump
    )]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}