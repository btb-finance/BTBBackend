use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, TokenAccount, Token, Mint},
    associated_token::AssociatedToken
};

declare_id!("teyk2AYGjb6SGPtXxqr6EW1bqQNtthfjaQggMYDSSXv");

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
        // Amount validation
        require!(amount > 0, CustomError::InvalidAmount);
        // Token type validation
        require!(token_type >= 1 && token_type <= 3, CustomError::InvalidTokenType);
        
        let btb_sale_account = &ctx.accounts.btb_sale_account;
        
        // Calculate payment amount
        let stored_price = btb_sale_account.btb_price;
        let payment_amount = (amount as u128)
            .checked_mul(stored_price as u128)
            .ok_or(CustomError::CalculationError)?
            .checked_div(1_000)
            .ok_or(CustomError::CalculationError)? as u64;
        
        // Add minimum amount check if needed
        require!(
            payment_amount >= 1_000, // Minimum 0.001 USDT/USDC/PayPal (6 decimals)
            CustomError::AmountTooSmall
        );
        
        // Verify token type and mint
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
    
        // Transfer payment tokens from user to owner
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.owner_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            payment_amount,  // Changed from total_price to payment_amount
        )?;
        
        // Transfer BTB tokens to user
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
            amount,
        )?;
        
        Ok(())
    }
    
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = signer, space = 8 + 32 * 6 + 8 * 2,   
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
}