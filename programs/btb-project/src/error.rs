use anchor_lang::prelude::*;

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

    #[msg("Claiming is not available yet.")]
    ClaimNotAvailableYet,

    #[msg("There is nothing to claim.")]
    NothingToClaim,



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
 