use anchor_lang::{prelude::*,  solana_program::bpf_loader_upgradeable};
use crate::initialize_data_account::InitializeDataAccount;

#[derive(Accounts)]
pub struct UpdateData<'info> {
    #[account(
        mut,
        seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()],
        bump
    )]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
     /// CHECK: Program data account containing upgrade authority
     #[account(
        constraint = program_data.owner == &bpf_loader_upgradeable::ID
    )]
    pub program_data: AccountInfo<'info>,
  
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}