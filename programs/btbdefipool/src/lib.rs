use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("22nJsqqdzzcKf1gLf4BBH4dw6pRWtdp4E4ndi6fweHY2");

#[program]
pub mod btbdefipool {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String) -> Result<()> {
        let investment = &mut ctx.accounts.investment;
        investment.name = name;
        investment.balance = 0;
        investment.owner = *ctx.accounts.user.key;
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> ProgramResult {
        let txn = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.investment.key(),
            amount
        );
        anchor_lang::solana_program::program::invoke(
            &txn,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.investment.to_account_info()
            ],
        )?;
        ctx.accounts.investment.balance += amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> ProgramResult {
        let investment = &mut ctx.accounts.investment;
        let user = &mut ctx.accounts.user;
        if investment.owner != user.key() {
            return Err(ProgramError::IncorrectProgramId);
        }
        let rent = Rent::get()?.minimum_balance(investment.to_account_info().data_len());
        if **investment.to_account_info().lamports.borrow() - rent < amount {
            return Err(ProgramError::InsufficientFunds);
        }
        **investment.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }
}

#[account]
pub struct Investment {
    pub name: String,
    pub balance: u64,
    pub owner: Pubkey,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer=user, space=5000, seeds=[b"investoraccount", user.key().as_ref()], bump)]
    pub investment: Account<'info, Investment>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info>{
    #[account(mut)]
    pub investment: Account<'info, Investment>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info>{
    #[account(mut)]
    pub investment: Account<'info, Investment>,
    #[account(mut)]
    pub user: Signer<'info>,
}

