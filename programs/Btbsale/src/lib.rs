use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use std::str::FromStr;

declare_id!("C2yzwzE7V5dmhRaWmxjNbvtFDNanRjmPUcDtKo6n6kHY");


pub const ADMIN_PUBKEY: &str = "b7doByN6f3VyrV26aVA3TeNUuMFJ7w22sbUDaK3QNx4";

#[program]
pub mod btb_token_sale {
    use super::*;

    pub fn initialize_sale(
        ctx: Context<InitializeSale>,
        btb_token_address: Pubkey,
        payment_token_addresses: Vec<Pubkey>,
        btb_team_wallet: Pubkey,
        token_price: u64,
        token_vesting_price: u64,
    ) -> Result<()> {
     
        let admin_pubkey = Pubkey::from_str(ADMIN_PUBKEY)
            .map_err(|_| CustomError::InvalidAdminAddress)?;
        require!(
            ctx.accounts.owner.key() == admin_pubkey,
            CustomError::UnauthorizedAdmin
        );

    
        require!(payment_token_addresses.len() <= 5, CustomError::TooManyPaymentTokens);
        require!(token_price > 0, CustomError::InvalidPrice);
        require!(token_vesting_price > 0, CustomError::InvalidPrice);
        
        let sale = &mut ctx.accounts.sale;
        
        let mut payment_tokens = Vec::new();
        for token_address in payment_token_addresses {
            payment_tokens.push(PaymentToken {
                mint: token_address,
                is_active: true,
            });
        }

        sale.btb_token_address = btb_token_address;
        sale.btb_team_wallet = btb_team_wallet;
        sale.token_price = token_price;
        sale.token_vesting_price = token_vesting_price;
        sale.owner = admin_pubkey;
        sale.payment_tokens = payment_tokens;
        sale.is_active = true;
        
        Ok(())
    }

    pub fn update_sale_status(
        ctx: Context<UpdateSale>,
        is_active: bool,
    ) -> Result<()> {
        let admin_pubkey = Pubkey::from_str(ADMIN_PUBKEY)
            .map_err(|_| CustomError::InvalidAdminAddress)?;
        require!(
            ctx.accounts.owner.key() == admin_pubkey,
            CustomError::UnauthorizedAdmin
        );

        let sale = &mut ctx.accounts.sale;
        sale.is_active = is_active;
        Ok(())
    }

    pub fn update_token_status(
        ctx: Context<UpdateSale>,
        token_index: u8,
        is_active: bool,
    ) -> Result<()> {
        let admin_pubkey = Pubkey::from_str(ADMIN_PUBKEY)
            .map_err(|_| CustomError::InvalidAdminAddress)?;
        require!(
            ctx.accounts.owner.key() == admin_pubkey,
            CustomError::UnauthorizedAdmin
        );

        let sale = &mut ctx.accounts.sale;
        require!(
            (token_index as usize) < sale.payment_tokens.len(),
            CustomError::InvalidPaymentToken
        );

        sale.payment_tokens[token_index as usize].is_active = is_active;
        Ok(())
    }

    pub fn buy_tokens(
        ctx: Context<BuyTokens>, 
        amount: u64,
        payment_token_index: u8
    ) -> Result<()> {
        let sale = &ctx.accounts.sale;
        
        
        require!(sale.is_active, CustomError::SaleInactive);
        
        require!(
            (payment_token_index as usize) < sale.payment_tokens.len(),
            CustomError::InvalidPaymentToken
        );

        let payment_token = &sale.payment_tokens[payment_token_index as usize];
        
        require!(payment_token.is_active, CustomError::InactivePaymentToken);
        require!(
            ctx.accounts.payment_token_mint.key() == payment_token.mint,
            CustomError::InvalidPaymentTokenMint
        );

        let total_cost = amount
            .checked_mul(sale.token_price)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .checked_div(10u64.pow(9))
            .ok_or(ProgramError::ArithmeticOverflow)?;

        require!(
            ctx.accounts.user_payment_token_account.amount >= total_cost,
            CustomError::InsufficientPaymentTokens
        );
        require!(
            ctx.accounts.sale_btb_account.amount >= amount,
            CustomError::InsufficientSaleTokens
        );

    
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.user_payment_token_account.to_account_info(),
                    to: ctx.accounts.team_payment_token_account.to_account_info(),
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

        msg!("Transfer successful");
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PaymentToken {
    pub mint: Pubkey,
    pub is_active: bool,
}

#[account]
pub struct Sale {
    pub btb_token_address: Pubkey,
    pub btb_team_wallet: Pubkey,
    pub owner: Pubkey,
    pub payment_tokens: Vec<PaymentToken>,
    pub token_price: u64,
    pub token_vesting_price: u64,
    pub is_active: bool,
}

#[derive(Accounts)]
pub struct InitializeSale<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 +  32 + 32 +  32 +  4 +   (32 + 1) * 5 + 8 +   
               8 +   
               1 +   
               64,   
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
pub struct UpdateSale<'info> {
    #[account(mut)]
    pub sale: Account<'info, Sale>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64, payment_token_index: u8)]
pub struct BuyTokens<'info> {
    #[account(mut, seeds = [b"sale"], bump)]
    pub sale: Account<'info, Sale>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_payment_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = btb_mint,
        associated_token::authority = user
    )]
    pub user_btb_account: Account<'info, TokenAccount>,
    pub btb_mint: Account<'info, Mint>,
    pub payment_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub team_payment_token_account: Account<'info, TokenAccount>,
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

#[error_code]
pub enum CustomError {
    #[msg("Too many payment tokens")]
    TooManyPaymentTokens,
    #[msg("Invalid payment token")]
    InvalidPaymentToken,
    #[msg("Invalid payment token mint")]
    InvalidPaymentTokenMint,
    #[msg("Insufficient payment tokens")]
    InsufficientPaymentTokens,
    #[msg("Insufficient sale tokens")]
    InsufficientSaleTokens,
    #[msg("Payment token is not active")]
    InactivePaymentToken,
    #[msg("Only admin can perform this action")]
    UnauthorizedAdmin,
    #[msg("Invalid admin address")]
    InvalidAdminAddress,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Sale is not active")]
    SaleInactive,
}