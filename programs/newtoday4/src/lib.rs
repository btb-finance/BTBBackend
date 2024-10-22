use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
declare_id!("C2yzwzE7V5dmhRaWmxjNbvtFDNanRjmPUcDtKo6n6kHY");

#[program]
pub mod btb_token_sale {
    use super::*;
    pub fn initialize_sale(
        ctx: Context<InitializeSale>,
        btb_token_address: Pubkey,
        usdc_token_address: Pubkey,
        btb_team_wallet: Pubkey,
        token_price: u64,
        token_vesting_price: u64,
    ) -> Result<()> {
        let sale = &mut ctx.accounts.sale;
        sale.btb_token_address = btb_token_address;
        sale.usdc_token_address = usdc_token_address;
        sale.btb_team_wallet = btb_team_wallet;
        sale.token_price = token_price;
        sale.token_vesting_price = token_vesting_price;
        sale.owner = ctx.accounts.owner.key();
        Ok(())
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
        let sale = &ctx.accounts.sale;
        let total_cost = amount.checked_mul(sale.token_price).ok_or(ProgramError::ArithmeticOverflow)?;

       
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_usdc_account.to_account_info(),
                    to: ctx.accounts.btb_team_wallet.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            total_cost,
        )?;

       
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.sale_btb_account.to_account_info(),
                    to: ctx.accounts.user_btb_account.to_account_info(),
                    authority: ctx.accounts.sale.to_account_info(),
                },
                &[&[b"sale", &[ctx.bumps.sale]]],
            ),
            amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeSale<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 32,
        seeds = [b"sale"],
        bump
    )]
    pub sale: Account<'info, Sale>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub btb_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = owner,
        associated_token::mint = btb_mint,
        associated_token::authority = sale
    )]
    pub sale_btb_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct BuyTokens<'info> {
    #[account(mut, seeds = [b"sale"], bump)]
    pub sale: Account<'info, Sale>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        constraint = user_usdc_account.mint == usdc_mint.key()
    )]
    pub user_usdc_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = btb_mint,
        associated_token::authority = user
    )]
    pub user_btb_account: Account<'info, TokenAccount>,
    pub btb_mint: Account<'info, Mint>,
    pub usdc_mint: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = sale.btb_team_wallet
    )]
    pub btb_team_wallet: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = btb_mint,
        associated_token::authority = sale
    )]
    pub sale_btb_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Sale {
    pub btb_token_address: Pubkey,
    pub usdc_token_address: Pubkey,
    pub btb_team_wallet: Pubkey,
    pub token_price: u64,
    pub token_vesting_price: u64,
    pub owner: Pubkey,
}