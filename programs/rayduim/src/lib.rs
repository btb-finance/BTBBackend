
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use std::str::FromStr;

declare_id!("82p44HjdpEBjebM2KBvP7seVB1RZFVEczBYaeSDUaa7v");

#[program]
pub mod raydium_liquidity {
    use super::*;

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_0: u64,
        amount_1: u64,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<()> {
        require!(tick_lower < tick_upper, ErrorCode::InvalidTickRange);
        require!(amount_0 > 0 && amount_1 > 0, ErrorCode::InvalidAmount);

        let accounts = vec![
            AccountMeta::new(ctx.accounts.pool_state.key(), false),
            AccountMeta::new(ctx.accounts.position_nft_mint.key(), false),
            AccountMeta::new(ctx.accounts.position_nft_account.key(), false),
            AccountMeta::new(ctx.accounts.token_account_0.key(), false),
            AccountMeta::new(ctx.accounts.token_account_1.key(), false),
            AccountMeta::new(ctx.accounts.token_vault_0.key(), false),
            AccountMeta::new(ctx.accounts.token_vault_1.key(), false),
            AccountMeta::new(ctx.accounts.tick_array_lower.key(), false),
            AccountMeta::new(ctx.accounts.tick_array_upper.key(), false),
            AccountMeta::new(ctx.accounts.owner.key(), true),
            AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
        ];

        let mut ix_data = vec![];
        ix_data.extend_from_slice(&[4]); // Instruction discriminator
        ix_data.extend_from_slice(&amount_0.to_le_bytes());
        ix_data.extend_from_slice(&amount_1.to_le_bytes());
        ix_data.extend_from_slice(&tick_lower.to_le_bytes());
        ix_data.extend_from_slice(&tick_upper.to_le_bytes());

        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.clmm_program.key(),
            accounts,
            data: ix_data,
        };

        let account_infos = [
            ctx.accounts.pool_state.to_account_info(),
            ctx.accounts.position_nft_mint.to_account_info(),
            ctx.accounts.position_nft_account.to_account_info(),
            ctx.accounts.token_account_0.to_account_info(),
            ctx.accounts.token_account_1.to_account_info(),
            ctx.accounts.token_vault_0.to_account_info(),
            ctx.accounts.token_vault_1.to_account_info(),
            ctx.accounts.tick_array_lower.to_account_info(),
            ctx.accounts.tick_array_upper.to_account_info(),
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ];

        anchor_lang::solana_program::program::invoke(
            &ix,
            &account_infos,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    /// Raydium CLMM program
    pub clmm_program: Program<'info, ClmmProgram>,
    
    /// Pool state account
    /// CHECK: This account is checked by the Raydium program
    #[account(mut)]
    pub pool_state: AccountInfo<'info>,
    
    /// Position NFT mint
    #[account(mut)]
    pub position_nft_mint: Account<'info, Mint>,
    
    /// Position NFT token account
    #[account(mut)]
    pub position_nft_account: Account<'info, TokenAccount>,
    
    /// User's token account for token 0
    #[account(mut)]
    pub token_account_0: Account<'info, TokenAccount>,
    
    /// User's token account for token 1
    #[account(mut)]
    pub token_account_1: Account<'info, TokenAccount>,
    
    /// Pool's vault for token 0
    #[account(mut)]
    pub token_vault_0: Account<'info, TokenAccount>,
    
    /// Pool's vault for token 1
    #[account(mut)]
    pub token_vault_1: Account<'info, TokenAccount>,
    
    /// Lower tick array account
    /// CHECK: This account is checked by the Raydium program
    #[account(mut)]
    pub tick_array_lower: AccountInfo<'info>,
    
    /// Upper tick array account
    /// CHECK: This account is checked by the Raydium program
    #[account(mut)]
    pub tick_array_upper: AccountInfo<'info>,
    
    /// Owner of the position
    /// CHECK: This is the signer that authorizes the operation
    #[account(mut, signer)]
    pub owner: AccountInfo<'info>,
    
    /// SPL Token program
    pub token_program: Program<'info, Token>,
}

#[derive(Clone)]
pub struct ClmmProgram;
impl anchor_lang::Id for ClmmProgram {
    fn id() -> Pubkey {
        Pubkey::from_str("devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH").unwrap()
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Lower tick must be less than upper tick")]
    InvalidTickRange,
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
}