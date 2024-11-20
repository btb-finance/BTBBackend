use anchor_lang::{prelude::*, solana_program::bpf_loader_upgradeable};
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};

declare_id!("jj96gvB1CD4ei2pRQbCvsB8uHK71aKv1uEKMCKURsMb");

#[program]
pub mod pda_vesting {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, 
        btb: Pubkey,
        usdt: Pubkey, 
        usdc: Pubkey, 
        paypal_usd: Pubkey, 
        owner_token_receive_wallet: Pubkey,
        btb_price: u64, 
        vesting_price: u64
    ) -> Result<()> {
        let program_data = ctx.accounts.program_data.try_borrow_data()?;
        let upgrade_authority = Pubkey::new_from_array(program_data[13..45].try_into().unwrap());
        require!(ctx.accounts.signer.key() == upgrade_authority, CustomError::UnauthorizedDeployer);
        
        require!(btb_price > 0, CustomError::ZeroBTBPrice);
        require!(vesting_price > 0, CustomError::ZeroVestingPrice);
        
        let sale_account = &mut ctx.accounts.btb_sale_account;
        sale_account.btb = btb;
        sale_account.usdt = usdt;
        sale_account.usdc = usdc;
        sale_account.paypal_usd = paypal_usd;
        sale_account.owner_token_receive_wallet = owner_token_receive_wallet;
        sale_account.owner_initialize_wallet = ctx.accounts.signer.key();
        sale_account.btb_price = btb_price;
        sale_account.vesting_price = vesting_price;
        sale_account.is_sale_active = true;
        Ok(())
    }

    pub fn toggle_sale(ctx: Context<UpdateData>) -> Result<()> {
        let program_data = ctx.accounts.program_data.try_borrow_data()?;
        let upgrade_authority = Pubkey::new_from_array(program_data[13..45].try_into().unwrap());
        require!(ctx.accounts.signer.key() == upgrade_authority, CustomError::UnauthorizedDeployer);

        let sale_account = &mut ctx.accounts.btb_sale_account;
        sale_account.is_sale_active = !sale_account.is_sale_active;
        Ok(())
    }

    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        let program_data = ctx.accounts.program_data.try_borrow_data()?;
        let upgrade_authority = Pubkey::new_from_array(program_data[13..45].try_into().unwrap());
        require!(ctx.accounts.signer.key() == upgrade_authority, CustomError::UnauthorizedDeployer);

        let btb_sale_account = &ctx.accounts.btb_sale_account;
        let balance = ctx.accounts.btb_sale_token_account.amount;
        require!(balance > 0, CustomError::NoTokensToWithdraw);

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.btb_sale_token_account.to_account_info(),
                    to: ctx.accounts.owner_btb_account.to_account_info(),
                    authority: ctx.accounts.btb_sale_account.to_account_info(),
                },
                &[&[
                    b"btb-sale-account",
                    btb_sale_account.owner_initialize_wallet.as_ref(),
                    &[ctx.bumps.btb_sale_account],
                ]],
            ),
            balance,
        )?;
        
        Ok(())
    }

    pub fn update_initialize(ctx: Context<UpdateData>,
         btb: Pubkey,
         usdt: Pubkey,
         usdc: Pubkey, 
         paypal_usd: Pubkey,
         owner_token_receive_wallet: Pubkey,
         btb_price: u64,
         vesting_price: u64
    ) -> Result<()> {
        let program_data = ctx.accounts.program_data.try_borrow_data()?;
        let upgrade_authority = Pubkey::new_from_array(program_data[13..45].try_into().unwrap());
        require!(ctx.accounts.signer.key() == upgrade_authority, CustomError::UnauthorizedDeployer);

        require!(btb_price > 0, CustomError::ZeroBTBPrice);
        require!(vesting_price > 0, CustomError::ZeroVestingPrice);

        let sale_account = &mut ctx.accounts.btb_sale_account;
        sale_account.btb = btb;
        sale_account.usdt = usdt;
        sale_account.usdc = usdc;
        sale_account.paypal_usd = paypal_usd;
        sale_account.owner_token_receive_wallet = owner_token_receive_wallet;
        sale_account.btb_price = btb_price;
        sale_account.vesting_price = vesting_price;
        Ok(())
    }

    pub fn buy_token(ctx: Context<BuyToken>, amount: u64, token_type: u8) -> Result<()> {
        require!(
            ctx.accounts.btb_sale_account.is_sale_active,
            CustomError::SaleNotActive
        );

        require!(amount > 0, CustomError::InvalidAmount);
        require!(token_type >= 1 && token_type <= 3, CustomError::InvalidTokenType);
        
        let btb_sale_account = &ctx.accounts.btb_sale_account;
        
        let stored_price = btb_sale_account.btb_price;
    
        let btb_amount = (amount as u128)
            .checked_mul(1_000_000_000)  
            .ok_or(CustomError::CalculationError)?
            .checked_div(stored_price as u128)  
            .ok_or(CustomError::CalculationError)? as u64;
    
        require!(
            amount >= 1_000, 
            CustomError::AmountTooSmall
        );
        
        let expected_mint = match token_type {
            1 => btb_sale_account.usdt,
            2 => btb_sale_account.usdc,
            3 => btb_sale_account.paypal_usd,
            _ => return Err(CustomError::InvalidTokenType.into()),
        };
        
        require!(
            ctx.accounts.user_token_account.mint == expected_mint,
            CustomError::InvalidTokenMint
        );
    
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.owner_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;
        
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.btb_sale_token_account.to_account_info(),
                    to: ctx.accounts.user_btb_account.to_account_info(),
                    authority: ctx.accounts.btb_sale_account.to_account_info(),
                },
                &[&[
                    b"btb-sale-account",
                    btb_sale_account.owner_initialize_wallet.as_ref(),
                    &[ctx.bumps.btb_sale_account],
                ]],
            ),
            btb_amount,
        )?;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + 32 * 6 + 8 * 2 + 1,
        seeds = [b"btb-sale-account", signer.key().as_ref()],
        bump
    )]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    #[account(
        init,
        payer = signer, 
        associated_token::mint = btb_mint_account,
        associated_token::authority = btb_sale_account
    )]
    pub btb_sale_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Program data account containing upgrade authority
    #[account(
        constraint = program_data.owner == &bpf_loader_upgradeable::ID
    )]
    pub program_data: AccountInfo<'info>,
    
    pub btb_mint_account: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(
        seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()],
        bump
    )]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    /// CHECK: Program data account containing upgrade authority
    #[account(
        constraint = program_data.owner == &bpf_loader_upgradeable::ID
    )]
    pub program_data: AccountInfo<'info>,
    
    #[account(
        mut,
        associated_token::mint = btb_mint_account,
        associated_token::authority = btb_sale_account
    )]
    pub btb_sale_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = owner_btb_account.mint == btb_mint_account.key(),
        constraint = owner_btb_account.owner == signer.key()
    )]
    pub owner_btb_account: Account<'info, TokenAccount>,
    
    pub btb_mint_account: Account<'info, Mint>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

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

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(
        seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()],
        bump
    )]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = owner_token_account.mint == user_token_account.mint,
        constraint = owner_token_account.owner == btb_sale_account.owner_token_receive_wallet
    )]
    pub owner_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = btb_mint_account,
        associated_token::authority = btb_sale_account
    )]
    pub btb_sale_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = btb_mint_account,
        associated_token::authority = user
    )]
    pub user_btb_account: Account<'info, TokenAccount>,
    
    pub btb_mint_account: Account<'info, Mint>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
#[derive(Default)]
pub struct InitializeDataAccount {
    pub btb: Pubkey,
    pub usdt: Pubkey,
    pub usdc: Pubkey,
    pub paypal_usd: Pubkey,
    pub owner_token_receive_wallet: Pubkey,
    pub owner_initialize_wallet: Pubkey,
    pub btb_price: u64,
    pub vesting_price: u64,
    pub is_sale_active: bool,
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized: Only owner can perform this action")]
    Unauthorized,
    
    #[msg("BTB price must be greater than 0")]
    ZeroBTBPrice,
    
    #[msg("Vesting price must be greater than 0")]
    ZeroVestingPrice,
    
    #[msg("Invalid token type selected")]
    InvalidTokenType,
    
    #[msg("Calculation overflow occurred")]
    CalculationError,
    
    #[msg("Invalid token mint address")]
    InvalidTokenMint,

    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    
    #[msg("Amount is too small. Minimum amount is 0.001 tokens")]
    AmountTooSmall,
    
    #[msg("Amount exceeds maximum limit")]
    AmountTooLarge,

    #[msg("Sale is not currently active")]
    SaleNotActive,

    #[msg("No tokens available to withdraw")]
    NoTokensToWithdraw,

    #[msg("Unauthorized: Only program deployer can initialize")]
    UnauthorizedDeployer,
}