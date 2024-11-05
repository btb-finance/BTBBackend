
use crate::CustomError;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::InitializeDataAccount::InitializeDataAccount;


     // Implement InitSpace to calculate space dynamically
     const INIT_SPACE: usize =    8 +  // discriminator
                                    32 * 6 + // 6 Pubkeys
                                    8 * 2;   // 2 u64s

    pub fn initialize(ctx: Context<Initialize>, 
        btb: Pubkey,
        usdt: Pubkey, // Company's usdt account address
        usdc: Pubkey, // Company's usdc account address
        paypal_usd: Pubkey,  // Company's paypal usd account address
        owner_token_receive_wallet: Pubkey,
        btb_price: u64
    ) -> Result<()> {

        require!(btb_price > 0, CustomError::ZeroBTBPrice);
       // require!(token_price < 0, CustomError::InvalidPrice);

        let sale_account = &mut ctx.accounts.btb_sale_account;
        sale_account.btb = btb;
        sale_account.usdt = usdt;
        sale_account.usdc = usdc;
        sale_account.paypal_usd = paypal_usd;
        sale_account.owner_token_receive_wallet = owner_token_receive_wallet;
        sale_account.owner_initialize_wallet = ctx.accounts.signer.key();
        sale_account.btb_price = btb_price;
        msg!("Sale initialization complete");
        Ok(())
    }

    #[derive(Accounts)]
    pub struct Initialize<'info> {

        #[account(init, 
                    payer = signer, 
                    space = INIT_SPACE,   
                    seeds = [b"btb-sale-account", signer.key().as_ref()],
                    bump)]
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

    /*
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
    */

