use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};

declare_id!("aaUSJAx9C6W8nQqdX1H4YibzBh17tXA8JZnuRqj8ukZ");

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
        sale_account.is_sale_active = true; // Sale active by default
        Ok(())
    }

    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        require!(new_admin != Pubkey::default(), CustomError::InvalidNewAdmin);
        
        let sale_account = &mut ctx.accounts.btb_sale_account;
        
        // Verify current signer is the admin
        require!(
            ctx.accounts.signer.key() == sale_account.owner_initialize_wallet,
            CustomError::Unauthorized
        );
    
        // Update the admin
        sale_account.owner_initialize_wallet = new_admin;
        
        Ok(())
    }

    pub fn toggle_sale(ctx: Context<UpdateData>) -> Result<()> {
        let sale_account = &mut ctx.accounts.btb_sale_account;
        
        // Only owner can toggle sale status
        require!(
            ctx.accounts.signer.key() == sale_account.owner_initialize_wallet,
            CustomError::Unauthorized
        );
        
        // Toggle the sale status
        sale_account.is_sale_active = !sale_account.is_sale_active;
        
        Ok(())
    }

    pub fn emergency_withdraw(ctx: Context<EmergencyWithdraw>) -> Result<()> {
        let btb_sale_account = &ctx.accounts.btb_sale_account;
        
        // Only owner can withdraw
        require!(
            ctx.accounts.signer.key() == btb_sale_account.owner_initialize_wallet,
            CustomError::Unauthorized
        );
        
        // Get the current balance of BTB tokens in sale account
        let balance = ctx.accounts.btb_sale_token_account.amount;
        
        // If balance is 0, return early
        require!(balance > 0, CustomError::NoTokensToWithdraw);

        // Transfer all BTB tokens to owner's wallet
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
            balance, // Transfer full balance
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
        require!(btb_price > 0, CustomError::ZeroBTBPrice);
        require!(vesting_price > 0, CustomError::ZeroVestingPrice);

        let sale_account = &mut ctx.accounts.btb_sale_account;
        require!(ctx.accounts.signer.key() == sale_account.owner_initialize_wallet, CustomError::Unauthorized);

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
        // Check if sale is active
        require!(
            ctx.accounts.btb_sale_account.is_sale_active,
            CustomError::SaleNotActive
        );

        require!(amount > 0, CustomError::InvalidAmount);
        require!(token_type >= 1 && token_type <= 3, CustomError::InvalidTokenType);
        
        let btb_sale_account = &ctx.accounts.btb_sale_account;
        
        let stored_price = btb_sale_account.btb_price;
    
        // Calculate BTB tokens to send to user
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
    #[account(init, payer = signer, space = 8 + 32 * 6 + 8 * 2 + 1,  // Added 1 for bool
              seeds = [b"btb-sale-account", signer.key().as_ref()], bump)]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    #[account(init, payer = signer, 
              associated_token::mint = btb_mint_account,
              associated_token::authority = btb_sale_account)]
    pub btb_sale_token_account: Account<'info, TokenAccount>,
    pub btb_mint_account: Account<'info, Mint>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(
        mut,
        seeds = [b"btb-sale-account", signer.key().as_ref()],
        bump
    )]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info> {
    #[account(seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()], bump)]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    
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
    #[account(mut, seeds = [b"btb-sale-account", signer.key().as_ref()], bump)]
    pub btb_sale_account: Account<'info, InitializeDataAccount>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(seeds = [b"btb-sale-account", btb_sale_account.owner_initialize_wallet.as_ref()], bump)]
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

    #[msg("Cannot transfer admin to zero address")]
    InvalidNewAdmin,
}