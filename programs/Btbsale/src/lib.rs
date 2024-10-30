use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("2zuQMgCNYgKzRJFuJqVEd8uzLCZ79CRYy8rSLub74iy1");

#[program]
pub mod btb_token_sale {
    use super::*;

    pub fn initialize_sale(
        ctx: Context<InitializeSale>,
        btb_token_address: Pubkey,
        payment_token_addresses: Vec<Pubkey>,
        btb_team_wallet: Pubkey,
        token_price: u64,  // Price in payment token's smallest unit (e.g., 1000 for 0.001 USDC)
        token_vesting_price: u64,
    ) -> Result<()> {
        require!(payment_token_addresses.len() <= 3, CustomError::TooManyPaymentTokens);
        require!(payment_token_addresses.len() > 0, CustomError::NoPaymentTokens);
        require!(token_price > 0, CustomError::InvalidPrice);
        require!(token_vesting_price > 0, CustomError::InvalidPrice);
        
        let sale = &mut ctx.accounts.sale;
        
        // Create and log payment tokens
        let payment_tokens: Vec<PaymentToken> = payment_token_addresses.iter()
            .map(|&addr| {
                let token = PaymentToken {
                    mint: addr,
                    is_active: true,
                    decimals: 6,  // Fixed decimals for USDC/USDT/PayPal
                };
                msg!("Adding payment token: {}", addr);
                token
            })
            .collect();

        msg!("Sale initialization parameters:");
        msg!("BTB Token Address: {}", btb_token_address);
        msg!("Team Wallet: {}", btb_team_wallet);
        msg!("Token Price: {}", token_price);
        msg!("Number of payment tokens: {}", payment_tokens.len());

        sale.btb_token_address = btb_token_address;
        sale.btb_team_wallet = btb_team_wallet;
        sale.token_price = token_price;
        sale.token_vesting_price = token_vesting_price;
        sale.owner = ctx.accounts.owner.key();
        sale.payment_tokens = payment_tokens;
        sale.is_active = true;

        msg!("Sale initialization complete");
        Ok(())
    }

    
    pub fn buy_tokens(
        ctx: Context<BuyTokens>, 
        amount: u64,  // Amount in BTB lamports (9 decimals)
        payment_token_index: u8
    ) -> Result<()> {
        let sale = &ctx.accounts.sale;
        
        require!(sale.is_active, CustomError::SaleInactive);
        
        // Validate and get payment token
        let payment_token_index = payment_token_index as usize;
        require!(
            payment_token_index < sale.payment_tokens.len(),
            CustomError::InvalidPaymentToken
        );

        let payment_token = &sale.payment_tokens[payment_token_index];
        require!(payment_token.is_active, CustomError::InactivePaymentToken);

        // Validate payment token addresses
        msg!("Payment token validation:");
        msg!("Index: {}", payment_token_index);
        msg!("Expected mint: {}", payment_token.mint);
        msg!("Provided mint: {}", ctx.accounts.payment_token_mint.key());
        msg!("Team wallet: {}", sale.btb_team_wallet);

        require!(
            ctx.accounts.payment_token_mint.key() == payment_token.mint,
            CustomError::InvalidPaymentTokenMint
        );

        // Validate team payment account
        require!(
            ctx.accounts.team_payment_token_account.mint == payment_token.mint &&
            ctx.accounts.team_payment_token_account.owner == sale.btb_team_wallet,
            CustomError::InvalidTeamPaymentAccount
        );

        // Calculate BTB price (1 BTB = 0.001 payment tokens)
        // BTB has 9 decimals, payment tokens have 6 decimals
        let total_cost = if amount >= 1_000_000_000 {
            // For whole BTB tokens (≥1 BTB)
            let btb_tokens = amount / 1_000_000_000;
            btb_tokens
                .checked_mul(sale.token_price)
                .ok_or(ProgramError::ArithmeticOverflow)?
        } else {
            // For fractional BTB amounts (<1 BTB)
            ((amount as u128) * sale.token_price as u128 / 1_000_000_000) as u64
        };

        // Log transaction details
        msg!("Transaction details:");
        msg!("BTB amount: {} ({} BTB)", amount, amount as f64 / 1_000_000_000.0);
        msg!("Payment amount: {} ({} tokens)", total_cost, total_cost as f64 / 1_000_000.0);
        msg!("User payment balance: {}", ctx.accounts.user_payment_token_account.amount);
        msg!("Sale BTB balance: {}", ctx.accounts.sale_btb_account.amount);

        // Validate balances
        require!(
            ctx.accounts.user_payment_token_account.amount >= total_cost,
            CustomError::InsufficientPaymentTokens
        );
        require!(
            ctx.accounts.sale_btb_account.amount >= amount,
            CustomError::InsufficientSaleTokens
        );

        // Execute payment token transfer
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

        // Execute BTB transfer
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

        msg!("Transfer successful:");
        msg!("BTB transferred: {} ({} BTB)", amount, amount as f64 / 1_000_000_000.0);
        msg!("Payment tokens transferred: {} ({} tokens)", total_cost, total_cost as f64 / 1_000_000.0);
        Ok(())
    }


    pub fn update_sale_status(
        ctx: Context<UpdateSale>,
        is_active: bool,
    ) -> Result<()> {
        require!(
            ctx.accounts.owner.key() == ctx.accounts.sale.owner,
            CustomError::UnauthorizedAdmin
        );

        let sale = &mut ctx.accounts.sale;
        sale.is_active = is_active;
        msg!("Sale status updated: {}", is_active);
        Ok(())
    }

    pub fn update_token_status(
        ctx: Context<UpdateSale>,
        token_index: u8,
        is_active: bool,
    ) -> Result<()> {
        require!(
            ctx.accounts.owner.key() == ctx.accounts.sale.owner,
            CustomError::UnauthorizedAdmin
        );

        let sale = &mut ctx.accounts.sale;
        require!(
            (token_index as usize) < sale.payment_tokens.len(),
            CustomError::InvalidPaymentToken
        );

        sale.payment_tokens[token_index as usize].is_active = is_active;
        msg!("Token {} status updated: {}", token_index, is_active);
        Ok(())
    }

    pub fn update_sale_params(
        ctx: Context<UpdateSale>,
        token_price: Option<u64>,
        token_vesting_price: Option<u64>,
        btb_team_wallet: Option<Pubkey>,
    ) -> Result<()> {
        require!(
            ctx.accounts.owner.key() == ctx.accounts.sale.owner,
            CustomError::UnauthorizedAdmin
        );

        let sale = &mut ctx.accounts.sale;

        // Update token price if provided
        if let Some(price) = token_price {
            require!(price > 0, CustomError::InvalidPrice);
            sale.token_price = price;
            msg!("Token price updated to: {}", price);
        }

        // Update vesting price if provided
        if let Some(price) = token_vesting_price {
            require!(price > 0, CustomError::InvalidPrice);
            sale.token_vesting_price = price;
            msg!("Token vesting price updated to: {}", price);
        }

        // Update team wallet if provided
        if let Some(wallet) = btb_team_wallet {
            sale.btb_team_wallet = wallet;
            msg!("Team wallet updated to: {}", wallet);
        }

        msg!("Sale parameters updated successfully");
        Ok(())
    }

    pub fn add_payment_token(
        ctx: Context<UpdateSale>,
        payment_token_mint: Pubkey,
    ) -> Result<()> {
        require!(
            ctx.accounts.owner.key() == ctx.accounts.sale.owner,
            CustomError::UnauthorizedAdmin
        );

        let sale = &mut ctx.accounts.sale;
        
        // Check if payment token already exists
        require!(
            !sale.payment_tokens.iter().any(|token| token.mint == payment_token_mint),
            CustomError::PaymentTokenExists
        );

        // Check maximum payment tokens limit
        require!(
            sale.payment_tokens.len() < 3,
            CustomError::TooManyPaymentTokens
        );

        // Add new payment token
        let new_token = PaymentToken {
            mint: payment_token_mint,
            is_active: true,
            decimals: 6, // Fixed decimals for USDC/USDT/PayPal
        };

        sale.payment_tokens.push(new_token);

        msg!("New payment token added: {}", payment_token_mint);
        msg!("Total payment tokens: {}", sale.payment_tokens.len());
        
        Ok(())
    }

    pub fn remove_payment_token(
        ctx: Context<UpdateSale>,
        token_index: u8,
    ) -> Result<()> {
        require!(
            ctx.accounts.owner.key() == ctx.accounts.sale.owner,
            CustomError::UnauthorizedAdmin
        );

        let sale = &mut ctx.accounts.sale;
        
        let index = token_index as usize;
        require!(
            index < sale.payment_tokens.len(),
            CustomError::InvalidPaymentToken
        );

        // Remove payment token
        let removed_token = sale.payment_tokens.remove(index);
        msg!("Payment token removed: {}", removed_token.mint);
        msg!("Remaining payment tokens: {}", sale.payment_tokens.len());

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PaymentToken {
    pub mint: Pubkey,
    pub is_active: bool,
    pub decimals: u8,
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
        space = 8 + 32 + 32 + 32 + 4 + (32 + 1 + 1) * 3 + 8 + 8 + 1 + 64,
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

    /// CHECK: Team wallet that will receive payments
    pub team_wallet: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
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
    
    #[account(
        mut,
        constraint = team_payment_token_account.mint == payment_token_mint.key(),
        constraint = team_payment_token_account.owner == sale.btb_team_wallet
    )]
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
    pub rent: Sysvar<'info, Rent>,
}

#[error_code]
pub enum CustomError {
    #[msg("Too many payment tokens")]
    TooManyPaymentTokens,
    
    #[msg("No payment tokens specified")]
    NoPaymentTokens,
    
    #[msg("Invalid payment token")]
    InvalidPaymentToken,
    
    #[msg("Invalid payment token mint")]
    InvalidPaymentTokenMint,
    
    #[msg("Invalid team payment token account")]
    InvalidTeamPaymentAccount,
    
    #[msg("Insufficient payment tokens")]
    InsufficientPaymentTokens,
    
    #[msg("Insufficient sale tokens")]
    InsufficientSaleTokens,
    
    #[msg("Payment token is not active")]
    InactivePaymentToken,
    
    #[msg("Only the sale owner can perform this action")]
    UnauthorizedAdmin,
    
    #[msg("Invalid price")]
    InvalidPrice,
    
    #[msg("Sale is not active")]
    SaleInactive,
    
    #[msg("Payment token already exists")]
    PaymentTokenExists,
    
    #[msg("Insufficient payment")]
    InsufficientPayment,
}
