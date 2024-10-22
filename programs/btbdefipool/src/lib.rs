// lib.rs
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use devai::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// DevAI configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DevAIConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[program]
pub mod solana_token_vesting {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, devai_config: DevAIConfig) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        vesting_account.authority = ctx.accounts.authority.key();
        vesting_account.bump = ctx.bumps.vesting_account;
        vesting_account.devai_config = devai_config;

        // Use DevAI to process the initialization
        let ai_result = devai::run("initialize", &[])?;
        msg!("DevAI processed result: {}", ai_result);

        Ok(())
    }

    // Other functions will be added in subsequent steps
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 1 + 64, // Space for DevAIConfig
        seeds = [b"vesting_account"],
        bump
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VestingAccount {
    pub authority: Pubkey,
    pub bump: u8,
    pub devai_config: DevAIConfig,
}

// Error enum will be added as we implement more functionality
#[error_code]
pub enum VestingError {
    #[msg("Vesting has not started yet")]
    VestingNotStarted,
    #[msg("Nothing to claim")]
    NothingToClaim,
}

// Helper function to use DevAI with the stored configuration
pub fn use_devai(vesting_account: &VestingAccount, prompt_name: &str, args: &[(&str, &str)]) -> Result<String> {
    devai::run_with_config(
        prompt_name,
        args,
        &devai::Config {
            model: vesting_account.devai_config.model.clone(),
            temperature: vesting_account.devai_config.temperature,
            max_tokens: vesting_account.devai_config.max_tokens,
            ..Default::default()
        },
    )
}