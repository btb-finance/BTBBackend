use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };

use crate::state::InitializeDataAccount::VestingAccount;
use crate::CustomError;

    pub fn create_user_vesting(
        ctx: Context<CreateUserAccount>,
        start_time: i64,
        end_time: i64,
        total_amount: i64,
        cliff_time: i64
    ) -> Result<()> {
        *ctx.accounts.user_account = UserAccount {
            beneficiary: ctx.accounts.beneficiary.key(),
            start_time,
            end_time,
            total_amount,
            total_withdrawn: 0,
            cliff_time,
            vesting_account: ctx.accounts.vesting_account.key(),
            bump: ctx.bumps.user_account,
        };

        Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>, _company_name: String) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let now = Clock::get()?.unix_timestamp;

        // Check if the current time is before the cliff time
        if now < user_account.cliff_time {
            return Err(CustomError::ClaimNotAvailableYet.into());
        }
        // Calculate the vested amount
        let time_since_start = now.saturating_sub(user_account.start_time);
        let total_vesting_time = user_account.end_time.saturating_sub(
            user_account.start_time
        );
        let vested_amount = if now >= user_account.end_time {
            user_account.total_amount
        } else {
            (user_account.total_amount * time_since_start) / total_vesting_time
        };

        //Calculate the amount that can be withdrawn
        let claimable_amount = vested_amount.saturating_sub(user_account.total_withdrawn);
        // Check if there is anything left to claim
        if claimable_amount == 0 {
            return Err(CustomError::NothingToClaim.into());
        }
        let transfer_cpi_accounts = TransferChecked {
            from: ctx.accounts.treasury_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.treasury_token_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let signer_seeds: &[&[&[u8]]] = &[
            &[
                b"vesting_treasury",
                ctx.accounts.vesting_account.company_name.as_ref(),
                &[ctx.accounts.vesting_account.treasury_bump],
            ],
        ];
        let cpi_context = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(
            signer_seeds
        );
        let decimals = ctx.accounts.mint.decimals;
        token_interface::transfer_checked(cpi_context, claimable_amount as u64, decimals)?;
        user_account.total_withdrawn += claimable_amount;
        Ok(())
    }

    #[account]
    #[derive(InitSpace, Debug)]
    pub struct UserAccount {
        pub beneficiary: Pubkey,
        pub start_time: i64,
        pub end_time: i64,
        pub total_amount: i64,
        pub total_withdrawn: i64,
        pub cliff_time: i64,
        pub vesting_account: Pubkey,
        pub bump: u8,
    }

    #[derive(Accounts)]
    pub struct CreateUserAccount<'info> {
        #[account(mut)]
        pub owner: Signer<'info>,
        pub beneficiary: SystemAccount<'info>,
        #[account(has_one = owner)]
        pub vesting_account: Account<'info, VestingAccount>,
        #[account(
            init,
            space = 8 + UserAccount::INIT_SPACE,
            payer = owner,
            seeds = [b"user_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
            bump
        )]
        pub user_account: Account<'info, UserAccount>,
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
#[instruction(company_name: String)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
        bump = user_account.bump,
        has_one = beneficiary,
        has_one = vesting_account
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        mut,
        seeds = [company_name.as_ref()],
        bump = vesting_account.bump,
        has_one = treasury_token_account,
        has_one = mint
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = beneficiary,
        associated_token::mint = mint,
        associated_token::authority = beneficiary,
        associated_token::token_program = token_program
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

