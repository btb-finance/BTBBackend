use crate::state::BTBVestingAccount

pub mod buy_at_vesting{

    pub fn create_user_vesting(
        ctx: Context<UserAccountDetail>,
        start_time: i64,
        end_time: i64,
        total_amount: i64,
        cliff_time: i64
    ) -> Result<()> {
        *ctx.accounts.user_account = UserVestingAccount {
            beneficiary: ctx.accounts.beneficiary.key(),
            start_time,
            end_time,
            total_amount,
            total_withdrawn: 0,
            cliff_time,
            vesting_account: ctx.accounts.vesting_account.key(),
            bump: ctx.bumps.employee_account,
        };

        Ok(())
    }


    pub fn claim_tokens(ctx: Context<ClaimTokens>, _company_name: String) -> Result<()> {
        let user_account = &mut ctx.accounts.user_account;
        let now = Clock::get()?.unix_timestamp;

        // Check if the current time is before the cliff time
        if now < employee_account.cliff_time {
            return Err(ErrorCode::ClaimNotAvailableYet.into());
        }
        // Calculate the vested amount
        let time_since_start = now.saturating_sub(employee_account.start_time);
        let total_vesting_time = employee_account.end_time.saturating_sub(
            employee_account.start_time
        );
        let vested_amount = if now >= employee_account.end_time {
            employee_account.total_amount
        } else {
            (employee_account.total_amount * time_since_start) / total_vesting_time
        };

        //Calculate the amount that can be withdrawn
        let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);
        // Check if there is anything left to claim
        if claimable_amount == 0 {
            return Err(ErrorCode::NothingToClaim.into());
        }
        let transfer_cpi_accounts = TransferChecked {
            from: ctx.accounts.treasury_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.employee_token_account.to_account_info(),
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
        employee_account.total_withdrawn += claimable_amount;
        Ok(())
    }


    #[derive(Accounts)]
    pub struct UserAccountDetail<'info> {
        #[account(mut)]
        pub owner: Signer<'info>,
        pub beneficiary: SystemAccount<'info>,
        #[account(has_one = owner)]
        pub vesting_account: Account<'info, BTBVestingAccount>,
        #[account(
            init,
            space = 8 + UserVestingAccount::INIT_SPACE,
            payer = owner,
            seeds = [b"user_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
            bump
        )]
        pub user_account: Account<'info, UserVestingAccount>,
        pub system_program: Program<'info, System>,
    }
    
}