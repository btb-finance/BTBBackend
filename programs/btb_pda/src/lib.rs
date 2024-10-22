use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint, Transfer};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("9biQAHwmNpe995VHh9K6KT43hxQY1gzy7UNs1a3irr7v");

#[program]
pub mod btb_pda {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        btb_token: Pubkey,
        btb_price: u64,
        vesting_price: u64,
    ) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        pda_account.btb_token = btb_token;
        pda_account.owner_wallet = ctx.accounts.owner.key();
        pda_account.btb_price = btb_price;
        pda_account.vesting_price = vesting_price;

        Ok(())
    }

    pub fn buy_token(ctx: Context<BuyToken>, sol_amount: u64 , token_amount : u64) -> Result<()> {
        let pda_account = &ctx.accounts.pda_account;
        

        // Transfer SOL from user to owner
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &pda_account.owner_wallet,
            sol_amount
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.owner_wallet.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Transfer tokens from PDA's associated token account to user's associated token account
        let seeds = &[
            b"btb-account",
            pda_account.owner_wallet.as_ref(),
            &[ctx.bumps.pda_account],
        ];
        let signer = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pda_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.pda_account.to_account_info(),
                },
                signer,
            ),
            token_amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 32 + 8 + 8,
        seeds = [b"btb-account", owner.key().as_ref()],
        bump
    )]
    pub pda_account: Account<'info, PdaAccount>,
    
    #[account(
        init,
        payer = owner,
        associated_token::mint = btb_token,
        associated_token::authority = pda_account
    )]
    pub pda_token_account: Account<'info, TokenAccount>,
    
    pub btb_token: Account<'info, Mint>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(
        seeds = [b"btb-account", pda_account.owner_wallet.key().as_ref()],
        bump,
    )]
    pub pda_account: Account<'info, PdaAccount>,

    #[account(
        mut,
        associated_token::mint = btb_token,
        associated_token::authority = pda_account
    )]
    pub pda_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = btb_token,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub btb_token: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: This is the owner wallet stored in the PDA account
    #[account(mut, address = pda_account.owner_wallet)]
    pub owner_wallet: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct PdaAccount {
    pub btb_token: Pubkey,
    pub owner_wallet: Pubkey,
    pub btb_price: u64,
    pub vesting_price: u64,
}
